#!/usr/bin/env nu
# Block until the host is free-ish, then exit 0 so a caller can start a latency-sensitive run.
# (e.g. `direnv exec . scripts/wait-for-free-ish-host.nu --load-per-cpu 1.25 --samples 3`).
#
# "Free-ish", not idle: this machine hosts many agent workspaces and is never quiet, so a gate
# demanding quiet never opens. Defaults are permissive; the question is oversubscription, not
# busyness. Load is per CPU so one threshold means the same on any core count.
#
# Three criteria, because each catches what the others miss. Memory is judged by headroom and by
# rate: a machine can hold free gigabytes while reclaiming hard the whole time, and it is that
# sustained paging, not the momentary figure, that lands in a latency measurement. Consecutive
# samples are required because load here is bursty, and one reading under the ceiling is usually the
# trough between two compiles.
#
# All input is /proc. Malformed input is an error, never a default, since a missing field would
# otherwise become a passing sample.
#
# Exit contract. Zero means the host was admitted. Without `--refusal-exit-code` every other exit is
# an error, refusal included, which is what the visible-rows probe has always relied on. With it, a
# completed deadline refusal — and only that — exits with the caller's code, so a caller can tell a
# gate that reached a verdict from one that never did. Every parse, argument, `/proc`, clock, page
# size and counter failure keeps raising, because none of them measured the host.

# One-minute load average, from /proc/loadavg.
def read-load-1 []: nothing -> float {
    let raw = (open --raw /proc/loadavg | str trim)
    let fields = ($raw | split row " " | where {|field| $field != "" })
    if ($fields | length) < 3 {
        error make { msg: $"malformed /proc/loadavg: expected at least 3 fields, got '($raw)'" }
    }
    let one = ($fields | first)
    try {
        $one | into float
    } catch {
        error make { msg: $"malformed /proc/loadavg: 1-minute field '($one)' is not a number" }
    }
}

# Online CPU count, from /proc/cpuinfo.
def read-cpu-count []: nothing -> int {
    let count = (open --raw /proc/cpuinfo | lines | where {|line| $line | str starts-with "processor" } | length)
    if $count < 1 {
        error make { msg: "malformed /proc/cpuinfo: no processor entries found" }
    }
    $count
}

# Available memory in GiB. MemAvailable, not MemFree: cache is reclaimable, so MemFree alone reports
# a healthy machine as starved.
def read-available-gib []: nothing -> float {
    let matched = (open --raw /proc/meminfo | lines | where {|line| $line | str starts-with "MemAvailable:" })
    if ($matched | is-empty) {
        error make { msg: "malformed /proc/meminfo: no MemAvailable line" }
    }
    let fields = ($matched | first | split row " " | where {|field| $field != "" })
    if ($fields | length) < 2 {
        error make { msg: $"malformed /proc/meminfo: unreadable MemAvailable line '($matched | first)'" }
    }
    let amount = ($fields | get 1)
    let kib = (try {
        $amount | into float
    } catch {
        error make { msg: $"malformed /proc/meminfo: MemAvailable value '($amount)' is not a number" }
    })
    $kib / 1048576.0
}

# Page size in bytes, to convert /proc/vmstat's page counters. Read, not assumed: this host is
# aarch64, where the page size is a kernel build option.
def read-page-bytes []: nothing -> int {
    let raw = (^getconf PAGESIZE | str trim)
    let bytes = (try {
        $raw | into int
    } catch {
        error make { msg: $"unreadable page size: getconf PAGESIZE said '($raw)'" }
    })
    if $bytes < 1 {
        error make { msg: $"implausible page size ($bytes) from getconf PAGESIZE" }
    }
    $bytes
}

# Pages swapped in and out since boot. Cumulative, so a rate needs two readings.
def read-swap-pages []: nothing -> record<pages_in: int, pages_out: int> {
    let counters = (open --raw /proc/vmstat | lines | parse "{key} {value}")
    let readings = ["pswpin" "pswpout"] | each {|name|
        let matched = ($counters | where key == $name)
        if ($matched | is-empty) {
            error make { msg: $"malformed /proc/vmstat: no ($name) counter" }
        }
        let value = ($matched | first | get value)
        try {
            $value | into int
        } catch {
            error make { msg: $"malformed /proc/vmstat: ($name) value '($value)' is not a number" }
        }
    }
    { pages_in: ($readings | first), pages_out: ($readings | last) }
}

def main [
    # Accepted 1-minute load average per CPU. 1.0 means "as many runnable threads as cores".
    --load-per-cpu: float = 1.5
    # Absolute 1-minute load ceiling. Overrides --load-per-cpu when given.
    --load-max: float
    # Refuse to start below this much available memory. Low by default: this host runs chronically
    # pressured, and a floor it never clears is a gate that never opens.
    --min-available-gib: float = 2.0
    # Refuse to start while swapping faster than this, combining swap-in and swap-out.
    --max-swap-io-mib-per-sec: float = 12.0
    # Seconds between samples.
    --interval-seconds: float = 15.0
    # Consecutive passing samples required before returning.
    --samples: int = 2
    # Give up once this long has elapsed, checked once per sample, so the deadline can be reached up
    # to one interval late. Unset waits indefinitely.
    --timeout-minutes: float
    # Exit with this status when the deadline is reached, rather than raising. Supplied by the caller
    # rather than fixed here so the refusal code has exactly one definition, in the caller that has
    # to decode it. Unset keeps the historical raise-on-refusal behaviour.
    --refusal-exit-code: int
] {
    if $samples < 1 {
        error make { msg: $"--samples must be at least 1, got ($samples)" }
    }
    if $interval_seconds <= 0.0 {
        error make { msg: $"--interval-seconds must be positive, got ($interval_seconds)" }
    }
    # Zero would make a refusal indistinguishable from admission, and a status outside a byte is not
    # a status a caller can observe.
    if $refusal_exit_code != null and ($refusal_exit_code < 1 or $refusal_exit_code > 255) {
        error make { msg: $"--refusal-exit-code must be between 1 and 255, got ($refusal_exit_code)" }
    }

    let cpus = (read-cpu-count)
    let ceiling = if $load_max == null { $load_per_cpu * $cpus } else { $load_max }
    if $ceiling <= 0.0 {
        error make { msg: $"computed load ceiling must be positive, got ($ceiling)" }
    }

    # Guarded, not just rounded: an interval that rounds to zero would divide the swap delta by a
    # window of no length and report an arbitrarily large rate.
    let interval_ms = ($interval_seconds * 1000.0 | math round | into int)
    if $interval_ms < 1 {
        error make { msg: $"--interval-seconds ($interval_seconds) is below the 1ms resolution of the sampling window" }
    }
    let interval = ($interval_ms * 1ms)
    let deadline = if $timeout_minutes == null {
        null
    } else {
        (date now) + (($timeout_minutes * 60.0 | math round | into int) * 1sec)
    }

    let ceiling_shown = ($ceiling | math round --precision 2)
    print $"waiting for a free-ish host: ($cpus) CPUs, 1-min load <= ($ceiling_shown), available RAM >= ($min_available_gib) GiB, swap I/O <= ($max_swap_io_mib_per_sec) MiB/s, ($samples) consecutive samples every ($interval_seconds)s"

    let page_bytes = (read-page-bytes)
    mut passing = 0
    mut sample = 0
    mut previous = { at: (date now), swap: (read-swap-pages) }
    loop {
        # The sleep leads: it is the window the swap delta is measured over, not a pause before one.
        sleep $interval
        $sample = $sample + 1

        let now = (date now)
        let swap = (read-swap-pages)
        let seconds = (($now - $previous.at) / 1sec)
        if $seconds <= 0.0 {
            error make { msg: $"non-advancing clock: sample ($sample) spanned ($seconds) seconds" }
        }
        let swapped_pages = (($swap.pages_in - $previous.swap.pages_in) + ($swap.pages_out - $previous.swap.pages_out))
        if $swapped_pages < 0 {
            error make { msg: $"/proc/vmstat swap counters went backwards by ($swapped_pages) pages; the machine cannot be measured across a counter reset" }
        }
        $previous = { at: $now, swap: $swap }
        let swap_mib_per_sec = ($swapped_pages * $page_bytes) / $seconds / 1048576.0

        let load1 = (read-load-1)
        let available_gib = (read-available-gib)
        let load_ok = ($load1 <= $ceiling)
        let ram_ok = ($available_gib >= $min_available_gib)
        let swap_ok = ($swap_mib_per_sec <= $max_swap_io_mib_per_sec)
        let all_ok = ($load_ok and $ram_ok and $swap_ok)

        $passing = if $all_ok { $passing + 1 } else { 0 }

        let load_shown = ($load1 | math round --precision 2)
        let per_cpu_shown = ($load1 / $cpus | math round --precision 2)
        let ram_shown = ($available_gib | math round --precision 1)
        let swap_shown = ($swap_mib_per_sec | math round --precision 1)
        let verdict = if $all_ok {
            "pass"
        } else if (not $load_ok) {
            "busy"
        } else if (not $swap_ok) {
            "swapping"
        } else {
            "low-ram"
        }
        print $"sample ($sample): load1=($load_shown) per-cpu=($per_cpu_shown) avail=($ram_shown)GiB swap=($swap_shown)MiB/s ($verdict) streak=($passing)/($samples)"

        if $passing >= $samples {
            print $"host is free-ish: 1-min load ($load_shown) over ($cpus) CPUs \(($per_cpu_shown) per CPU), ($ram_shown) GiB available, ($swap_shown) MiB/s swap I/O, after ($sample) samples"
            return
        }

        if $deadline != null and (date now) >= $deadline {
            # One message, reported two ways: raised when no refusal code was supplied, so the
            # historical caller sees exactly what it always saw, and written to stderr before the
            # agreed exit when one was, so the refusal is still diagnosable at the same wording.
            let refusal = $"timed out after ($sample) samples waiting for load <= ($ceiling_shown), RAM >= ($min_available_gib) GiB and swap I/O <= ($max_swap_io_mib_per_sec) MiB/s; last sample was load1=($load_shown), avail=($ram_shown)GiB, swap=($swap_shown)MiB/s"
            if $refusal_exit_code == null {
                error make { msg: $refusal }
            }
            print -e $refusal
            exit $refusal_exit_code
        }
    }
}
