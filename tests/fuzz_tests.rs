// tests/fuzz_tests.rs - Mutation fuzzing of the parser and roller
//
// Every 2.0.0 bug was one of four kinds a reviewer keeps missing and a machine
// finds: a panic (`1d6 + 1d4000000000`), a hang (`d6s4294967295`), an oversized
// reply (an error quoting a long expression twice), or a silently dropped
// number (`99999999999d6` rolling one die). This builds expressions by mutating
// corpus lines (boundary numbers, spliced tokens, extra math, glued modifiers)
// and checks each for the first three. Dropped numbers are covered by
// `ignored_numbers.snap`, which has the known gaps to compare against.
//
// Debug builds trap integer overflow, so arithmetic that would wrap silently in
// the release binary shows up here as a panic.
//
// Run on demand, it is not part of `cargo test`:
//     cargo test --test fuzz_tests -- --ignored --nocapture
//     FUZZ_ITERS=200000 FUZZ_SEED=7 cargo test --test fuzz_tests -- --ignored --nocapture
// Each finding prints its expression and roll seed, so it reproduces exactly.

use dicemaiden_rs::{
    commands::roll::error_reply, dice::rng::create_seeded_rng, format_multiple_results_with_limit,
    parse_and_roll_with_rng,
};
use rand::{Rng, RngExt};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const DISCORD_MESSAGE_LIMIT: usize = 2000;
/// A roll slower than this in a debug build is reported. Debug runs the dice
/// code roughly 10-30x slower than the release binary.
const SLOW: Duration = Duration::from_millis(250);
/// A roll still running after this is abandoned and reported as a hang. Its
/// thread keeps spinning, so the run stops after a few.
const HANG: Duration = Duration::from_secs(5);
const MAX_HANGS: usize = 3;

/// Numbers at and around every limit the parser enforces, and the integer
/// widths a count or side might be parsed into. The large values that do not
/// overflow find uncapped loops: those hang where the 32-bit edges overflow.
const BOUNDARIES: [&str; 24] = [
    "0",
    "1",
    "2",
    "9",
    "10",
    "20",
    "99",
    "100",
    "101",
    "499",
    "500",
    "501",
    "999",
    "1000",
    "1001",
    "32768",
    "65536",
    "1000000",
    "100000000",
    "2147483647",
    "2147483648",
    "4294967295",
    "4294967296",
    "99999999999",
];
const OPERATORS: [&str; 4] = ["+", "-", "*", "/"];

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Kind {
    Panic,
    Hang,
    Slow,
    OverLimit,
    EmptyResult,
}

struct Finding {
    kind: Kind,
    expression: String,
    roll_seed: u64,
    detail: String,
}

fn pick<'a>(rng: &mut impl Rng, items: &[&'a str]) -> &'a str {
    items[rng.random_range(0..items.len())]
}

/// Replace one run of digits in `token` with a boundary number, if it has any.
fn swap_number(rng: &mut impl Rng, token: &str) -> String {
    let runs: Vec<(usize, usize)> = {
        let mut runs = Vec::new();
        let mut start = None;
        for (i, c) in token.char_indices().chain([(token.len(), ' ')]) {
            match (c.is_ascii_digit(), start) {
                (true, None) => start = Some(i),
                (false, Some(s)) => {
                    runs.push((s, i));
                    start = None;
                }
                _ => {}
            }
        }
        runs
    };
    if runs.is_empty() {
        return token.to_string();
    }
    let (start, end) = runs[rng.random_range(0..runs.len())];
    format!(
        "{}{}{}",
        &token[..start],
        pick(rng, &BOUNDARIES),
        &token[end..]
    )
}

/// A random operand: a pool token, a boundary number, or a boundary-sized die.
fn operand(rng: &mut impl Rng, pool: &[&str]) -> String {
    match rng.random_range(0..3) {
        0 => pick(rng, pool).to_string(),
        1 => pick(rng, &BOUNDARIES).to_string(),
        _ => format!("{}d{}", pick(rng, &BOUNDARIES), pick(rng, &BOUNDARIES)),
    }
}

fn mutate(rng: &mut impl Rng, lines: &[&str], pool: &[&str]) -> String {
    let mut tokens: Vec<String> = pick(rng, lines)
        .split_whitespace()
        .map(str::to_string)
        .collect();
    for _ in 0..rng.random_range(1..=3) {
        let at = rng.random_range(0..=tokens.len());
        match rng.random_range(0..7) {
            0 | 1 if !tokens.is_empty() => {
                let i = rng.random_range(0..tokens.len());
                tokens[i] = swap_number(rng, &tokens[i]);
            }
            2 => tokens.insert(at, pick(rng, pool).to_string()),
            3 => {
                tokens.push(pick(rng, &OPERATORS).to_string());
                tokens.push(operand(rng, pool));
            }
            4 if !tokens.is_empty() => {
                let i = rng.random_range(0..tokens.len());
                tokens.remove(i);
            }
            5 if !tokens.is_empty() => {
                // Glue a modifier onto its neighbour: `4d6` + `k3` -> `4d6k3`
                let i = rng.random_range(0..tokens.len());
                let glued = format!("{}{}", tokens[i], pick(rng, pool));
                tokens[i] = glued;
            }
            _ => {
                tokens.push(";".to_string());
                tokens.extend(pick(rng, lines).split_whitespace().map(str::to_string));
            }
        }
    }
    tokens.join(" ")
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "non-string panic".to_string())
}

/// Roll and format `expression` on its own thread, so a hang can be timed out.
/// Returns the reply text, or what went wrong.
fn check(expression: &str, roll_seed: u64) -> Result<(String, Duration), (Kind, String)> {
    let (send, receive) = mpsc::channel();
    let owned = expression.to_string();
    std::thread::spawn(move || {
        let start = Instant::now();
        let outcome = std::panic::catch_unwind(|| {
            match parse_and_roll_with_rng(&owned, &mut create_seeded_rng(roll_seed)) {
                Ok(results) if results.is_empty() => Err((Kind::EmptyResult, String::new())),
                Ok(results) => Ok(format_multiple_results_with_limit(&results)),
                Err(e) => Ok(error_reply("Fuzzer", &owned, &e.to_string())),
            }
        });
        let outcome = match outcome {
            Ok(Ok(reply)) => Ok((reply, start.elapsed())),
            Ok(Err(failure)) => Err(failure),
            Err(payload) => Err((Kind::Panic, panic_message(payload.as_ref()))),
        };
        let _ = send.send(outcome);
    });
    receive
        .recv_timeout(HANG)
        .unwrap_or_else(|_| Err((Kind::Hang, format!("still running after {HANG:?}"))))
}

/// Group findings by kind and by the expression with its numbers blanked, so
/// one bug hit a thousand ways reads as one line with a count.
fn report(findings: &[Finding], iterations: u64, seed: u64) -> String {
    let mut groups: BTreeMap<(Kind, String), (usize, &Finding)> = BTreeMap::new();
    for finding in findings {
        let shape: String = finding
            .expression
            .split_whitespace()
            .map(|t| {
                let mut out = String::new();
                for c in t.chars() {
                    if !c.is_ascii_digit() {
                        out.push(c);
                    } else if !out.ends_with('N') {
                        out.push('N');
                    }
                }
                out
            })
            .collect::<Vec<_>>()
            .join(" ");
        groups
            .entry((finding.kind, shape))
            .and_modify(|(count, _)| *count += 1)
            .or_insert((1, finding));
    }
    let mut out = format!(
        "fuzzed {iterations} expressions (FUZZ_SEED={seed}): {} findings in {} shapes\n",
        findings.len(),
        groups.len()
    );
    for ((kind, shape), (count, example)) in &groups {
        out += &format!(
            "\n{kind:?} x{count}  {shape}\n  e.g. `{}` (roll seed {})\n  {}\n",
            example.expression, example.roll_seed, example.detail
        );
    }
    out
}

fn env_or(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[test]
#[ignore = "run on demand: cargo test --test fuzz_tests -- --ignored --nocapture"]
fn fuzz_parse_and_roll() {
    let iterations = env_or("FUZZ_ITERS", 20_000);
    let seed = env_or("FUZZ_SEED", 1);

    let corpus = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/corpus/expressions.txt"
    ))
    .expect("read corpus");
    let lines: Vec<&str> = corpus
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let pool: Vec<&str> = lines
        .iter()
        .flat_map(|l| l.split_whitespace())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    let mut rng = create_seeded_rng(seed);
    let mut seen = BTreeSet::new();
    let mut findings = Vec::new();
    let mut slowest: Vec<(Duration, String)> = Vec::new();

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    for roll_seed in 0..iterations {
        let expression = mutate(&mut rng, &lines, &pool);
        if !seen.insert(expression.clone()) {
            continue;
        }
        let failure = match check(&expression, roll_seed) {
            Ok((reply, elapsed)) => {
                slowest.push((elapsed, expression.clone()));
                if reply.len() > DISCORD_MESSAGE_LIMIT {
                    Some((Kind::OverLimit, format!("{} bytes", reply.len())))
                } else if elapsed > SLOW {
                    Some((Kind::Slow, format!("{elapsed:?}")))
                } else {
                    None
                }
            }
            Err(failure) => Some(failure),
        };
        if let Some((kind, detail)) = failure {
            findings.push(Finding {
                kind,
                expression,
                roll_seed,
                detail,
            });
            if findings.iter().filter(|f| f.kind == Kind::Hang).count() >= MAX_HANGS {
                break;
            }
        }
    }
    std::panic::set_hook(default_hook);

    slowest.sort_by_key(|(elapsed, _)| std::cmp::Reverse(*elapsed));
    let mut summary = report(&findings, iterations, seed);
    summary += "\nslowest rolls:\n";
    for (elapsed, expression) in slowest.iter().take(5) {
        summary += &format!("  {elapsed:?}  `{expression}`\n");
    }
    eprintln!("{summary}");
    assert!(findings.is_empty(), "{summary}");
}
