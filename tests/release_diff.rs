// tests/release_diff.rs - Roll dump for tools/diff-vs-release.sh
//
// Not a regression test on its own, so it is #[ignore]d. The script copies this
// file into a checkout of an older release, runs it there and here over the same
// expressions, and reports every result that differs. Keep it to API that old
// releases have: `parse_and_roll_with_rng`, `create_seeded_rng`,
// `format_multiple_results`.
//
// The corpus alone misses shapes nobody wrote down (`4 + 4d1 + 1 f1` was one),
// so it is widened two ways: every corpus line with numbers and math around it,
// and a grid of modifier families with math before and after each.

use dicemaiden_rs::{
    dice::rng::create_seeded_rng, format_multiple_results, parse_and_roll_with_rng,
};
use std::collections::BTreeSet;
use std::fmt::Write as _;

mod common;
use common::with_leading;

const SEEDS: [u64; 2] = [1, 42];
const LEADING: [&str; 5] = ["", "4 + ", "10 - ", "2 * ", "100/"];
const TRAILING: [&str; 4] = ["", " + 3", " - 1", " * 2"];
const BASES: [&str; 3] = ["4d1", "4d10", "1d20"];
const MODIFIERS: [&str; 17] = [
    "", "t7", "tl3", "f1", "b1", "t7 f1", "t7 b1", "t7ds10", "k2", "kl2", "e10", "ie10", "r1",
    "d1", "adv1", "mnm", "ww",
];
const MATH: [&str; 6] = ["", "+ 1", "- 1", "* 2", "/ 2", "+ 1d4"];

fn expressions(corpus: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in corpus.lines().map(str::trim).filter(|l| !l.is_empty()) {
        for lead in LEADING {
            // After `! comment`, trailing math would become comment text.
            let trailing: &[&str] = if line.contains('!') { &[""] } else { &TRAILING };
            for trail in trailing {
                out.insert(format!("{}{trail}", with_leading(line, lead)));
            }
        }
    }
    for base in BASES {
        for modifier in MODIFIERS {
            for math in MATH {
                for lead in LEADING {
                    out.insert(format!("{lead}{base} {math} {modifier}"));
                    out.insert(format!("{lead}{base} {modifier} {math}"));
                }
            }
        }
    }
    // Collapse the doubled spaces left by empty grid cells.
    out.into_iter()
        .map(|e| e.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect()
}

#[test]
#[ignore = "run by tools/diff-vs-release.sh"]
fn dump_rolls() {
    let env = |name: &str| std::env::var(name).unwrap_or_else(|_| panic!("{name} not set"));
    let corpus = std::fs::read_to_string(env("RELEASE_DIFF_CORPUS")).expect("read corpus");

    // Old releases panic on some inputs (`3d20 ww` before 1.8.2); record that
    // as a result instead of aborting the dump. The hook is silenced only while
    // rolling, so a failure to read or write still reports why.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let mut out = String::new();
    for expression in expressions(&corpus) {
        for seed in SEEDS {
            let roll = || match parse_and_roll_with_rng(&expression, &mut create_seeded_rng(seed)) {
                Ok(results) => format!("OK\t{}", format_multiple_results(&results)),
                Err(e) => format!("ERR\t{e}"),
            };
            let rendered =
                std::panic::catch_unwind(roll).unwrap_or_else(|_| "ERR\tPANIC".to_string());
            let _ = writeln!(
                out,
                "{expression}\t{seed}\t{}",
                rendered.replace('\n', "\\n")
            );
        }
    }
    std::panic::set_hook(default_hook);
    std::fs::write(env("RELEASE_DIFF_OUT"), out).expect("write output");
}
