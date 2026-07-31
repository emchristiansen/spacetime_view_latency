mod generated;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, ValueEnum};
use generated::append_message_reducer::append_message;
use generated::DbConnection;
use spacetimedb_sdk::DbContext;

const DATABASE: &str = "view-latency";
const BATCH_SIZE: u64 = 1000;
const NUM_BATCHES: u64 = 10;
const BATCH_DELAY_MS: u64 = 100;

#[derive(Parser)]
struct Cli {
    /// What to subscribe to: view, table, or none
    #[arg(long, value_enum, default_value = "view")]
    subscribe_to: SubscribeTo,

    /// Server URL
    #[arg(long, default_value = "http://127.0.0.1:3000")]
    server: String,

    /// Disable confirmed reads (reduces latency but sacrifices durability guarantees)
    #[arg(long)]
    no_confirmed_reads: bool,

    /// How writes are issued within a batch
    #[arg(long, value_enum, default_value = "burst")]
    write_mode: WriteMode,

    /// Append each rung to this file as its batch completes, so a run killed partway still leaves
    /// the rungs it did finish
    #[arg(long)]
    progress_path: Option<PathBuf>,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum SubscribeTo {
    /// Subscribe to messages_view
    View,
    /// Subscribe directly to messages table
    Table,
    /// No subscription (baseline reducer latency)
    None,
}

/// Whether a batch's writes are issued back-to-back or one at a time.
///
/// The two answer different questions. A burst measures how fast the server drains a pipeline that
/// is already full, so its per-write time is dominated by queueing. Pacing keeps exactly one write
/// outstanding, so what it records is the round trip itself.
#[derive(Debug, Copy, Clone, ValueEnum)]
enum WriteMode {
    /// Issue every write in the batch without awaiting. The committed default.
    Burst,
    /// Await each write's completion before issuing the next.
    Paced,
}

/// How one paced write finished, reported by its own callback.
///
/// Every arm of the callback sends one of these, so the issuing loop learns about a failure the
/// moment it happens. Signalling only on success would leave a failed write indistinguishable from
/// a slow one until the receive timeout elapsed, and would report it as a timeout.
enum WriteOutcome {
    /// The reducer committed; carries the round trip the issuing loop should record.
    Committed(Duration),
    /// The reducer ran and returned an error.
    ReducerFailed(String),
    /// The SDK could not complete the call.
    Internal(String),
}

/// Tracks reducer round-trip latencies
struct RoundtripTimer {
    latencies: Vec<Duration>,
}

impl RoundtripTimer {
    fn new() -> Self {
        Self {
            latencies: Vec::new(),
        }
    }

    fn record(&mut self, duration: Duration) {
        self.latencies.push(duration);
    }

    fn stats(&self) -> LatencyStats {
        if self.latencies.is_empty() {
            return LatencyStats::default();
        }

        let mut sorted: Vec<_> = self.latencies.iter().copied().collect();
        sorted.sort();

        let sum: Duration = sorted.iter().sum();
        let avg = sum / sorted.len() as u32;
        let p50 = sorted[sorted.len() / 2];
        let p99 = sorted[(sorted.len() as f64 * 0.99) as usize];

        LatencyStats { avg, p50, p99 }
    }

    fn clear(&mut self) {
        self.latencies.clear();
    }

    fn completed_count(&self) -> usize {
        self.latencies.len()
    }
}

#[derive(Default)]
struct LatencyStats {
    avg: Duration,
    p50: Duration,
    p99: Duration,
}

/// The ladder's column headings.
///
/// Rendered here rather than at each use so the streamed progress artifact and the final summary
/// cannot drift into two different tables.
fn ladder_header() -> String {
    format!(
        "{:>15} {:>12} {:>12} {:>12}",
        "Total messages", "Avg (ms)", "P50 (ms)", "P99 (ms)"
    )
}

/// One rung: the row for a batch that has completed.
fn ladder_row(total_messages: u64, stats: &LatencyStats) -> String {
    format!(
        "{:>10} {:>12.2} {:>12.2} {:>12.2}",
        total_messages,
        stats.avg.as_secs_f64() * 1000.0,
        stats.p50.as_secs_f64() * 1000.0,
        stats.p99.as_secs_f64() * 1000.0,
    )
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    run_test(&cli)
}

fn run_test(cli: &Cli) -> Result<()> {
    let subscribe_to = cli.subscribe_to;

    println!("=== View Latency Scaling Test ===");
    println!("Server: {}", cli.server);
    println!("Subscribe to: {:?}", subscribe_to);
    println!("Confirmed reads: {}", !cli.no_confirmed_reads);
    println!("Write mode: {:?}", cli.write_mode);
    println!("Batch size: {}, Batches: {}", BATCH_SIZE, NUM_BATCHES);
    println!();

    let roundtrip_timer = Arc::new(Mutex::new(RoundtripTimer::new()));

    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<()>();
    let (batch_done_tx, batch_done_rx) = std::sync::mpsc::channel::<()>();
    let (write_done_tx, write_done_rx) = std::sync::mpsc::channel::<WriteOutcome>();

    // Build connection
    let mut builder = DbConnection::builder()
        .with_uri(&cli.server)
        .with_database_name(DATABASE);

    if cli.no_confirmed_reads {
        builder = builder.with_confirmed_reads(false);
    }

    let conn = builder
        .on_connect({
            let ready_tx = ready_tx.clone();
            move |ctx, _identity, _token| match subscribe_to {
                SubscribeTo::View => {
                    ctx.subscription_builder()
                        .on_applied({
                            let ready_tx = ready_tx.clone();
                            move |_ctx| {
                                println!("Subscription applied (view)");
                                let _ = ready_tx.send(());
                            }
                        })
                        .on_error(|_ctx, err| {
                            eprintln!("Subscription error: {:?}", err);
                        })
                        .subscribe(["SELECT * FROM messages_view"]);
                }
                SubscribeTo::Table => {
                    ctx.subscription_builder()
                        .on_applied({
                            let ready_tx = ready_tx.clone();
                            move |_ctx| {
                                println!("Subscription applied (table)");
                                let _ = ready_tx.send(());
                            }
                        })
                        .on_error(|_ctx, err| {
                            eprintln!("Subscription error: {:?}", err);
                        })
                        .subscribe(["SELECT * FROM messages"]);
                }
                SubscribeTo::None => {
                    println!("No subscription");
                    let _ = ready_tx.send(());
                }
            }
        })
        .on_connect_error(|_ctx, err| {
            eprintln!("Connection error: {:?}", err);
        })
        .build()?;

    conn.run_threaded();

    ready_rx.recv()?;
    println!();

    // The summary is printed only once every batch is done, so a run killed at a caller's cap would
    // otherwise report nothing at all. When a progress path is given, each rung is appended and
    // flushed the moment its batch completes, and the file is the record of exactly how far the
    // ladder got. Final stdout is unaffected either way.
    let mut progress = match &cli.progress_path {
        Some(path) => {
            let mut file = File::create(path).with_context(|| {
                format!("could not create the progress file {}", path.display())
            })?;
            writeln!(file, "{}", ladder_header())
                .with_context(|| format!("could not write to {}", path.display()))?;
            file.flush()
                .with_context(|| format!("could not flush {}", path.display()))?;
            Some(file)
        }
        None => None,
    };

    // Run batches
    let mut all_stats: Vec<(u64, LatencyStats)> = Vec::new();

    for batch in 1..=NUM_BATCHES {
        std::thread::sleep(Duration::from_millis(BATCH_DELAY_MS));
        let total_messages = batch * BATCH_SIZE;

        {
            let mut timer = roundtrip_timer.lock().unwrap();
            timer.clear();
        }

        match cli.write_mode {
            WriteMode::Burst => {
                for i in 0..BATCH_SIZE {
                    let content = format!("batch{}_message{}", batch, i);
                    let start = Instant::now();
                    conn.reducers.append_message_then(content, {
                        let timer = roundtrip_timer.clone();
                        let done_tx = batch_done_tx.clone();
                        move |_ctx, result| match result {
                            Ok(Ok(())) => {
                                let mut t = timer.lock().unwrap();
                                t.record(start.elapsed());
                                if t.completed_count() as u64 >= BATCH_SIZE {
                                    let _ = done_tx.send(());
                                }
                            }
                            Ok(Err(err)) => eprintln!("Reducer failed: {err}"),
                            Err(err) => eprintln!("Internal error: {err:?}"),
                        }
                    })?;
                }

                batch_done_rx.recv_timeout(Duration::from_secs(60))?;
            }
            WriteMode::Paced => {
                for i in 0..BATCH_SIZE {
                    let content = format!("batch{}_message{}", batch, i);
                    let start = Instant::now();
                    conn.reducers.append_message_then(content, {
                        let done_tx = write_done_tx.clone();
                        move |_ctx, result| {
                            let outcome = match result {
                                Ok(Ok(())) => WriteOutcome::Committed(start.elapsed()),
                                Ok(Err(err)) => WriteOutcome::ReducerFailed(err),
                                Err(err) => WriteOutcome::Internal(format!("{err:?}")),
                            };
                            let _ = done_tx.send(outcome);
                        }
                    })?;

                    // Exactly one write is outstanding, so the next is not issued until this one is
                    // accounted for. A failure arrives as its own outcome rather than as silence.
                    let elapsed = match write_done_rx.recv_timeout(Duration::from_secs(60)) {
                        Ok(WriteOutcome::Committed(elapsed)) => elapsed,
                        Ok(WriteOutcome::ReducerFailed(err)) => {
                            bail!("paced write {i} of batch {batch} failed in the reducer: {err}")
                        }
                        Ok(WriteOutcome::Internal(err)) => {
                            bail!("paced write {i} of batch {batch} failed in the SDK: {err}")
                        }
                        Err(err) => {
                            bail!("paced write {i} of batch {batch} never completed: {err}")
                        }
                    };

                    let mut timer = roundtrip_timer.lock().map_err(|_| {
                        anyhow!("the round-trip timer was poisoned by a panicking callback")
                    })?;
                    timer.record(elapsed);
                }
            }
        }

        let stats = {
            let timer = roundtrip_timer.lock().unwrap();
            timer.stats()
        };

        if let Some(file) = progress.as_mut() {
            writeln!(file, "{}", ladder_row(total_messages, &stats))
                .context("could not append a completed rung to the progress file")?;
            // Flushed per rung, not at exit: the point is to survive being killed.
            file.flush()
                .context("could not flush a completed rung to the progress file")?;
        }

        all_stats.push((total_messages, stats));
    }

    // Summary
    println!("=== SUMMARY ===");
    println!("{}", ladder_header());
    for (total, stats) in &all_stats {
        println!("{}", ladder_row(*total, stats));
    }

    Ok(())
}
