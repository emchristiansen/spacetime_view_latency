//! The running, `/proc`-proven isolated standalone server capability.

use std::fs::{self, File};
use std::net::{TcpListener, TcpStream};
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, ensure, Context, Error, Result};
use spacetimedb_sdk::Identity;

use crate::manifest::database_identity::DatabaseIdentity;
use crate::manifest::listen_address::ListenAddress;
use crate::manifest::server_pid::ServerPid;
use crate::manifest::verified_module_artifact::VerifiedModuleArtifact;
use crate::manifest::verified_server_process::VerifiedServerProcess;
use crate::provision::fresh_data_dir::FreshDataDir;
use crate::provision::staged_module_wasm::StagedModuleWasm;
use crate::provision::teardown::{cleanup_data_dir, into_error, reap_child};
use crate::provision::verified_distribution::VerifiedDistribution;

/// `spacetimedb-standalone` subcommand and flags. A cross-process contract with the pinned
/// binary, verified against `start --help` and `v2.6.1`
/// `crates/standalone/src/subcommands/start.rs`; `--jwt-key-dir` is a hidden flag that
/// auto-generates the JWT keypair in the given directory.
const START_SUBCOMMAND: &str = "start";
const NON_INTERACTIVE_FLAG: &str = "--non-interactive";
const LISTEN_ADDR_FLAG: &str = "--listen-addr";
const DATA_DIR_FLAG: &str = "--data-dir";
const JWT_KEY_DIR_FLAG: &str = "--jwt-key-dir";

/// `spacetimedb-cli publish` subcommand and flags. A nameless anonymous publish to a fresh data
/// directory always *creates* a new database (no naming collision, no destructive delete, no
/// login/config dependence), verified against `publish --help` and `v2.6.1`
/// `crates/cli/src/subcommands/publish.rs`.
const PUBLISH_SUBCOMMAND: &str = "publish";
const BIN_PATH_FLAG: &str = "--bin-path";
const SERVER_FLAG: &str = "--server";
const ANONYMOUS_FLAG: &str = "--anonymous";
const YES_FLAG: &str = "--yes";
/// Ignore any ambient `spacetime.json` so the publish stdout grammar is independent of the
/// working directory (source: `publish --help`; `v2.6.1` `publish.rs:390` gates config loading
/// to `None` when set). This deterministically suppresses the conditional
/// `Using configuration from …` line.
const NO_CONFIG_FLAG: &str = "--no-config";

/// The complete, ordered stdout of a *nameless, fresh, `--bin-path`, local, `--no-config`*
/// publish, from `v2.6.1` `crates/cli/src/subcommands/publish.rs` — exactly these four lines
/// (each terminated by a single `\n`), no others:
///   1. `(WASM) Skipping build. Instead we are publishing <bin-path>` (`:542`, dynamic path).
///   2. `Uploading to <server> => <database_host>` (`:579`; `database_host = get_host_url` =
///      `proto://host` reconstructed from the server URL, `config.rs:551-553`, so it equals the
///      passed server URL).
///   3. `Publishing module...` (`:629`).
///   4. `Created new database with identity: <identity>` (`:192`; `domain = None` for a nameless
///      create, `client-api/routes/database.rs:864-865`, → `op = Created`), where `identity` is
///      `Identity`'s `Display` — its canonical 64-char lowercase hex (`to_hex`).
///
/// The `Publishing module … to database` line (`:498-506`) is gated on `using_config`, false
/// under `--no-config`; the `Updated`/named forms are structurally impossible for a nameless
/// fresh create and so cannot appear as line 4.
const PUBLISH_SKIP_BUILD_PREFIX: &str = "(WASM) Skipping build. Instead we are publishing ";
const PUBLISH_UPLOADING_PREFIX: &str = "Uploading to ";
const PUBLISH_UPLOADING_SEPARATOR: &str = " => ";
const PUBLISH_MODULE_LINE: &str = "Publishing module...";
const PUBLISH_CREATED_IDENTITY_PREFIX: &str = "Created new database with identity: ";
/// Exact number of stdout lines the ordered grammar above accounts for.
const EXPECTED_PUBLISH_LINE_COUNT: usize = 4;

/// Canonical identity hex length: a 32-byte SpacetimeDB identity as lowercase hex.
const IDENTITY_HEX_LEN: usize = 64;

/// Readiness polling against an **absolute** deadline: probe the listen socket until it connects
/// or `READINESS_DEADLINE` elapses. Each *requested* connect timeout and retry delay is capped by
/// the time remaining until the deadline; once control observes that the deadline has expired, no
/// new wait is started. A requested wait may itself return late, so the actual wall-clock return
/// may overshoot the deadline due to scheduler delay.
const READINESS_DEADLINE: Duration = Duration::from_secs(30);
const READINESS_POLL_INTERVAL: Duration = Duration::from_millis(100);
const READINESS_CONNECT_TIMEOUT: Duration = Duration::from_millis(500);

/// A live isolated standalone server whose running process has been proven (via
/// `/proc/<pid>/exe`) to be the verified pinned `spacetimedb-standalone`, owning its child
/// process and fresh data directory.
///
/// A liveness capability: it gates publication/measurement and is torn down only by an
/// explicit consuming [`Self::shutdown`]. [`Drop`] asserts both owned resources were handed
/// off, so a silent leak or best-effort cleanup is impossible. [`Self::start`] constructs the
/// asserting `Self` **only after** the `/proc` proof succeeds; any earlier failure tears the
/// raw child + data directory down explicitly, so no loud [`Drop`] runs during unwind.
#[derive(Debug)]
pub(crate) struct RunningPinnedServer {
    /// `Some` while running; taken by [`Self::shutdown`]. `Drop` requires it to be `None`.
    child: Option<Child>,
    /// `Some` while owned; taken by [`Self::shutdown`]. `Drop` requires it to be `None`.
    data_dir: Option<FreshDataDir>,
    listen: ListenAddress,
    process: VerifiedServerProcess,
}

impl RunningPinnedServer {
    /// Start the pinned standalone on `listen` with a fresh data directory, wait until it is
    /// ready, and prove its running process is the verified standalone via `/proc/<pid>/exe`.
    ///
    /// Teardown is centralized: the fresh data directory is loud from the moment it is created,
    /// so every fallible step after that point routes its error(s) — together with an explicit
    /// child-reap (if spawned) and data-directory cleanup — through [`into_error`], and the
    /// asserting `Self` is built only on the fully-proven success path.
    pub(crate) fn start(
        distribution: &VerifiedDistribution,
        listen: ListenAddress,
    ) -> Result<Self> {
        // No owned resource yet — a plain `?` is safe here (spec: "check the selected listen
        // address is free"). `--non-interactive` closes the residual TOCTOU window server-side.
        ensure_listen_addr_free(listen)?;

        let data_dir = FreshDataDir::new()?;

        // From here `data_dir` is loud; before a child exists only it must be cleaned.
        let mut child = match spawn_standalone(distribution, listen, &data_dir) {
            Ok(child) => child,
            Err(primary) => {
                let mut errors = vec![primary];
                cleanup_data_dir(data_dir, &mut errors);
                return Err(into_error(errors));
            }
        };

        // A child now exists; every remaining failure must reap it *and* clean the directory.
        // `bring_up` returns the primary readiness/prove fact(s) — including any log-read
        // failures accumulated as separate errors — and teardown then adds its own.
        match bring_up(distribution, listen, &data_dir, &mut child) {
            Ok(process) => Ok(Self {
                child: Some(child),
                data_dir: Some(data_dir),
                listen,
                process,
            }),
            Err(mut errors) => {
                reap_child(child, &mut errors);
                cleanup_data_dir(data_dir, &mut errors);
                Err(into_error(errors))
            }
        }
    }

    /// Publish the staged, hash-verified WASM to this server (nameless + anonymous) and record
    /// the resulting database identity by parsing the exact source-grounded success grammar.
    ///
    /// Takes `&self`: a failure here does not drop the server — the driver owns teardown.
    pub(crate) fn publish(
        &self,
        distribution: &VerifiedDistribution,
        staged: &StagedModuleWasm,
    ) -> Result<VerifiedModuleArtifact> {
        let server_url = self.listen.client_url();
        let output = Command::new(distribution.cli_exe())
            .arg(PUBLISH_SUBCOMMAND)
            .arg(BIN_PATH_FLAG)
            .arg(staged.bin_path())
            .arg(SERVER_FLAG)
            .arg(&server_url)
            .arg(ANONYMOUS_FLAG)
            .arg(YES_FLAG)
            .arg(NO_CONFIG_FLAG)
            .output()
            .with_context(|| {
                format!("running {:?} {PUBLISH_SUBCOMMAND}", distribution.cli_exe())
            })?;

        // Operational failure: lossy rendering is fine for diagnostics only.
        ensure!(
            output.status.success(),
            "publish exited unsuccessfully ({});\nstdout (lossy):\n{}\nstderr (lossy):\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );

        // Grammar is parsed from a *strict* UTF-8 decode; raw/lossy bytes are kept only for the
        // operational error path.
        let stdout = String::from_utf8(output.stdout).map_err(|e| {
            let lossy = String::from_utf8_lossy(e.as_bytes()).into_owned();
            anyhow!(
                "publish stdout is not valid UTF-8 ({}); lossy stdout:\n{lossy}",
                e.utf8_error()
            )
        })?;

        let identity = parse_published_identity(&stdout, staged.bin_path(), &server_url)
            .with_context(|| format!("parsing publish stdout:\n{stdout}"))?;
        Ok(VerifiedModuleArtifact::new(
            staged.sha256(),
            DatabaseIdentity::new(identity),
        ))
    }

    /// Explicitly shut the server down: reap the child (killing only if still running) and
    /// remove its fresh data directory, attempting *both* even if one fails and aggregating
    /// every error. Consumes `self`, taking both owned resources out *before* any fallible
    /// step, so [`Drop`] sees them handed off and cannot double-panic.
    pub(crate) fn shutdown(mut self) -> Result<()> {
        let child = self
            .child
            .take()
            .expect("RunningPinnedServer::shutdown called after the child was already taken");
        let data_dir = self
            .data_dir
            .take()
            .expect("RunningPinnedServer::shutdown called after the data dir was already taken");

        let mut errors = Vec::new();
        reap_child(child, &mut errors);
        cleanup_data_dir(data_dir, &mut errors);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(into_error(errors))
        }
    }

    /// The `/proc`-proven server process facts.
    pub(crate) fn process(&self) -> &VerifiedServerProcess {
        &self.process
    }

    /// The listen address this server was bound to.
    pub(crate) fn listen(&self) -> ListenAddress {
        self.listen
    }

    /// The server's fresh data directory.
    pub(crate) fn data_dir(&self) -> &Path {
        self.data_dir
            .as_ref()
            .expect("RunningPinnedServer::data_dir accessed after shutdown took the data dir")
            .data_dir()
    }

    /// The server's ephemeral JWT key directory (path only).
    pub(crate) fn keys_dir(&self) -> &Path {
        self.data_dir
            .as_ref()
            .expect("RunningPinnedServer::keys_dir accessed after shutdown took the data dir")
            .keys_dir()
    }
}

impl Drop for RunningPinnedServer {
    fn drop(&mut self) {
        assert!(
            self.child.is_none() && self.data_dir.is_none(),
            "RunningPinnedServer dropped without an explicit shutdown(); the server process \
             and/or data directory would leak"
        );
    }
}

/// Fail fast if `listen` is already bound (spec edge case). Binds then immediately releases;
/// `--non-interactive` makes the server itself fail if the port is taken in the interim.
fn ensure_listen_addr_free(listen: ListenAddress) -> Result<()> {
    let addr = listen.socket_addr();
    let probe = TcpListener::bind(addr)
        .with_context(|| format!("listen address {addr} is not free (bind probe failed)"))?;
    drop(probe);
    Ok(())
}

/// Spawn the pinned standalone with stdout/stderr redirected to files inside the owned tree (so
/// a full pipe buffer cannot deadlock the server), returning the live child.
fn spawn_standalone(
    distribution: &VerifiedDistribution,
    listen: ListenAddress,
    data_dir: &FreshDataDir,
) -> Result<Child> {
    let stdout_path = data_dir.server_stdout_path();
    let stderr_path = data_dir.server_stderr_path();
    let stdout = File::create(&stdout_path)
        .with_context(|| format!("creating server stdout capture file {stdout_path:?}"))?;
    let stderr = File::create(&stderr_path)
        .with_context(|| format!("creating server stderr capture file {stderr_path:?}"))?;

    Command::new(distribution.standalone_exe())
        .arg(START_SUBCOMMAND)
        .arg(NON_INTERACTIVE_FLAG)
        .arg(LISTEN_ADDR_FLAG)
        .arg(listen.socket_addr().to_string())
        .arg(DATA_DIR_FLAG)
        .arg(data_dir.data_dir())
        .arg(JWT_KEY_DIR_FLAG)
        .arg(data_dir.keys_dir())
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .with_context(|| {
            format!(
                "spawning {:?} {START_SUBCOMMAND}",
                distribution.standalone_exe()
            )
        })
}

/// Wait for readiness, then prove the running process is the verified standalone. Takes
/// `&mut child` so readiness polling can detect an early child exit. On failure returns the
/// primary fact(s) plus any separately-accumulated log-read failures, never one erasing another.
fn bring_up(
    distribution: &VerifiedDistribution,
    listen: ListenAddress,
    data_dir: &FreshDataDir,
    child: &mut Child,
) -> Result<VerifiedServerProcess, Vec<Error>> {
    await_ready(listen, data_dir, child)?;
    let pid = server_pid(child).map_err(|e| vec![e])?;
    VerifiedServerProcess::prove(pid, distribution.standalone_exe()).map_err(|e| vec![e])
}

/// Bounded readiness polling with child-exit detection against an **absolute** deadline: each
/// iteration first checks whether the child has already exited, then probes the listen socket,
/// with the connect timeout and inter-probe sleep each capped by the time remaining until the
/// deadline. Fails on child exit, on a poll error, or when the deadline elapses.
fn await_ready(
    listen: ListenAddress,
    data_dir: &FreshDataDir,
    child: &mut Child,
) -> Result<(), Vec<Error>> {
    let addr = listen.socket_addr();
    let deadline = Instant::now() + READINESS_DEADLINE;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(readiness_failure(
                    data_dir,
                    format!("server process exited during startup ({status})"),
                ));
            }
            Ok(None) => {}
            Err(e) => {
                return Err(vec![
                    Error::new(e).context("polling the server child during readiness wait")
                ]);
            }
        }

        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(deadline_exceeded(data_dir, addr));
        }
        // `remaining` is non-zero, so the capped connect timeout is non-zero (a zero Duration
        // would panic in `connect_timeout`).
        match TcpStream::connect_timeout(&addr, remaining.min(READINESS_CONNECT_TIMEOUT)) {
            Ok(stream) => {
                drop(stream);
                return Ok(());
            }
            Err(_) => {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    return Err(deadline_exceeded(data_dir, addr));
                }
                thread::sleep(remaining.min(READINESS_POLL_INTERVAL));
            }
        }
    }
}

/// The readiness-deadline failure, with captured logs folded in and any log-read failures
/// accumulated separately.
fn deadline_exceeded(data_dir: &FreshDataDir, addr: std::net::SocketAddr) -> Vec<Error> {
    readiness_failure(
        data_dir,
        format!("server did not become ready on {addr} within {READINESS_DEADLINE:?}"),
    )
}

/// Build the ordered error list for a readiness failure: the primary `fact` (with any *readable*
/// captured logs folded in for diagnosis), followed by a separate accumulated error for each
/// log-read *failure*. A diagnostic read failure therefore never erases the primary process
/// failure, and is never silently swallowed.
fn readiness_failure(data_dir: &FreshDataDir, fact: String) -> Vec<Error> {
    let mut read_failures = Vec::new();
    let stdout = read_captured_stream(data_dir.server_stdout_path(), &mut read_failures);
    let stderr = read_captured_stream(data_dir.server_stderr_path(), &mut read_failures);

    let mut message = fact;
    if let Some(contents) = stdout {
        message.push_str(&format!("\ncaptured server stdout:\n{contents}"));
    }
    if let Some(contents) = stderr {
        message.push_str(&format!("\ncaptured server stderr:\n{contents}"));
    }

    let mut errors = vec![anyhow!("{message}")];
    errors.extend(read_failures);
    errors
}

/// Read a captured log file for diagnostics, returning its contents on success or pushing a
/// distinct read-failure error (and returning `None`) on failure — never a placeholder.
fn read_captured_stream(path: PathBuf, read_failures: &mut Vec<Error>) -> Option<String> {
    match fs::read_to_string(&path) {
        Ok(contents) => Some(contents),
        Err(e) => {
            read_failures
                .push(Error::new(e).context(format!("reading captured server log {path:?}")));
            None
        }
    }
}

/// The spawned child's OS pid as a [`ServerPid`], failing fast on the impossible pid 0.
fn server_pid(child: &Child) -> Result<ServerPid> {
    let raw = child.id();
    let pid = NonZeroU32::new(raw).with_context(|| format!("server child reported pid {raw}"))?;
    Ok(ServerPid::new(pid))
}

/// Parse the database identity out of a *nameless, fresh, `--bin-path`, `--no-config`* publish's
/// stdout by matching the **complete ordered four-line grammar** (see the const block above),
/// not by scanning for a success line among arbitrary output. The whole stdout must be exactly:
///
/// 1. `(WASM) Skipping build. Instead we are publishing <staged bin-path>`
/// 2. `Uploading to <server_url> => <server_url>`
/// 3. `Publishing module...`
/// 4. `Created new database with identity: <canonical 64-char lowercase hex>`
///
/// each terminated by a single `\n`, with no carriage returns, no missing or extra terminal
/// newline, and no extra or reordered lines. Anything else — including the `Updated`/named
/// success forms, which are structurally impossible for a nameless fresh create and so must not
/// appear as line 4 — is a contract violation and is rejected.
fn parse_published_identity(
    stdout: &str,
    staged_bin_path: &Path,
    server_url: &str,
) -> Result<Identity> {
    // Strictly LF-terminated: a carriage return anywhere means the output is not the exact
    // grammar (reject CRLF outright rather than tolerating it).
    ensure!(
        !stdout.contains('\r'),
        "publish stdout contains a carriage return; expected strictly LF-terminated lines"
    );
    // Exactly one terminal newline: the output must end with `\n`, and once that single
    // terminator is stripped the body must not end with another (no extra/blank final line).
    let body = stdout
        .strip_suffix('\n')
        .context("publish stdout does not end with a terminal newline")?;
    ensure!(
        !body.ends_with('\n'),
        "publish stdout has an extra terminal newline (a trailing blank line)"
    );

    let lines: Vec<&str> = body.split('\n').collect();
    ensure!(
        lines.len() == EXPECTED_PUBLISH_LINE_COUNT,
        "expected exactly {EXPECTED_PUBLISH_LINE_COUNT} publish stdout lines, found {}",
        lines.len()
    );

    let expected_skip_build = format!("{PUBLISH_SKIP_BUILD_PREFIX}{}", staged_bin_path.display());
    ensure!(
        lines[0] == expected_skip_build,
        "publish stdout line 1 was {:?}, expected {:?}",
        lines[0],
        expected_skip_build
    );

    let expected_uploading =
        format!("{PUBLISH_UPLOADING_PREFIX}{server_url}{PUBLISH_UPLOADING_SEPARATOR}{server_url}");
    ensure!(
        lines[1] == expected_uploading,
        "publish stdout line 2 was {:?}, expected {:?}",
        lines[1],
        expected_uploading
    );

    ensure!(
        lines[2] == PUBLISH_MODULE_LINE,
        "publish stdout line 3 was {:?}, expected {PUBLISH_MODULE_LINE:?}",
        lines[2]
    );

    // Line 4: the prefix followed by exactly the identity (no trailing text), so the remainder
    // must be canonical lowercase hex with no surrounding whitespace.
    let hex = lines[3]
        .strip_prefix(PUBLISH_CREATED_IDENTITY_PREFIX)
        .with_context(|| {
            format!(
                "publish stdout line 4 was {:?}, expected {PUBLISH_CREATED_IDENTITY_PREFIX:?} \
             followed by the created database identity",
                lines[3]
            )
        })?;
    ensure!(
        is_canonical_identity_hex(hex),
        "published identity {hex:?} is not canonical {IDENTITY_HEX_LEN}-char lowercase hex"
    );
    Identity::from_hex(hex)
        .map_err(|e| anyhow!(e))
        .with_context(|| format!("decoding the published database identity {hex:?}"))
}

/// Whether `s` is exactly a canonical lowercase-hex SpacetimeDB identity (no `0x`, no padding,
/// no surrounding whitespace).
fn is_canonical_identity_hex(s: &str) -> bool {
    s.len() == IDENTITY_HEX_LEN
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
