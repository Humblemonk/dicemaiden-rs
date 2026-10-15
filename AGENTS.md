# dicemaiden-rs

Discord dice-rolling bot in Rust (Serenity), a rewrite of the Ruby DiceMaiden, running on
500,000+ servers. Single binary, SQLite for shard statistics, slash commands `/roll`, `/r`,
`/help`, `/purge`. Correctness and backward compatibility of dice syntax outweigh cleverness:
**a silently changed roll result is the worst possible bug.**

## Commands

```
cargo clippy -- -D warnings && cargo fmt --check && cargo test    # all must pass before done
UPDATE_SNAPSHOTS=1 cargo test --test snapshot_tests               # regenerate golden snapshots
npx jscpd@3 --threshold 0 --min-lines 5 --min-tokens 50 --max-lines 20000 --max-size 10mb src tests
tools/diff-vs-release.sh HEAD                                     # every roll, working tree vs last commit
cargo test --test fuzz_tests -- --ignored --nocapture             # mutation fuzz: panics, hangs, long replies
```

- Tests need no Discord token. For live testing, set `GUILD_ID` in `.env` so slash commands
  register instantly.
- CI's JSCPD check has threshold **0**: any clone of 5+ lines / 50+ tokens fails the build,
  in every language it scans, Markdown included. Keep the `--max-*` flags — without them JSCPD
  skips `roller.rs` and `parser.rs` and reports a clean run that CI will contradict.

## Architecture

Data flow: `dice/aliases.rs` (expands shorthand to a standard expression) → `dice/parser.rs`
(→ `Vec<DiceRoll>`) → `dice/roller.rs` (→ `Vec<RollResult>`) → `commands/roll.rs` (Discord
formatting). Core types live in `dice/mod.rs`; all randomness in `dice/rng.rs`.

## Parser Invariants (critical)

- Dice syntax is a public API. Never change the behavior of an existing expression unless
  explicitly asked; when in doubt, flag the compatibility question instead of deciding.
- The parser uses **prefix matching**. A new token that overlaps with or is a prefix of an
  existing one can silently break unrelated rolls — check every token in `roll_syntax.md` first.
- Multi-character prefixes (`ie`, `irg`, `ir`, `km`, `kl`, `tl`) must match **before** their
  single-character counterparts (`e`, `r`, `k`, `t`). The `COMBINED_MODIFIER_PATTERNS`,
  `SPLIT_MODIFIER_PATTERNS`, and `MODIFIER_START_PATTERNS` statics are ordered `Vec<Regex>`
  for this reason. Never sort or dedupe them.
- **Drop before explode**: dropped dice are never reconsidered for explosion. Intentional and
  tested; do not change modifier ordering semantics.
- **Where math applies depends on the modifier, and only the roller knows.** Before the parser
  moves or groups math around a modifier, read that modifier's arm in `roller.rs`. Never infer
  from a name (`ModifierKind::System`, `find_target_modifier_positions`): both 1.8.2
  regressions came from that. Verified behavior:

  | Roll | Math written before the modifier | Math written after it |
  |---|---|---|
  | `k` `kl` `d` `e` `ie` `r` `adv` | total | total |
  | `t` `tl` `t#ds#` 2d20 test (`Modifier::sets_success_count`) | each die | success count |
  | `f#` without `t#` | each die | ignored, leading number too (known gap) |
  | `b#` without `t#` | each die | total; ignored if math also comes before |
  | Between two targets | | ignored |
  | Other game systems | read the arm; `ignored_numbers.snap` lists those that ignore math | |

  A leading number (`4 + 4d10 t7`) goes on the success count of a success-counting roll, is
  rejected on an `f#`/`b#`-only roll with math before the target, is ignored on any other
  `f#`-only roll (`4 + 4d1 f1` shows -4), and otherwise goes in front of the user's math.

## Real-Play Check

Before fixing or extending how a roll behaves, ask: **would a player of that game system type
this roll in an actual session?** Answer from evidence, not instinct: the documented syntax
(`roll_syntax.md`, the help text, the original Ruby DiceMaiden readme) and how the system's
rulebook modifies a roll (flat numbers, a bigger pool, rerolls).

- **Passes**: documented syntax and alias forms, the system's own rolls with the modifiers its
  rules use, and mistakes players really make (a typo, math they'd expect to work). Fix it
  properly.
- **Fails**: internal tokens only aliases produce (`2d1 sw8`, `1d6 sil3`, `3d6 mme2`) and
  combinations the system gives no meaning (`k1` on a keep-highest pool, `t7` on VtM). Don't
  design behavior for it, and don't change what it does today. Only make sure it can't panic,
  hang or overflow (the fuzz test covers this), then list it as deferred.
- Borderline? Report the finding with its verdict and let the user decide before building.
  Code reviews should give each finding a verdict too.

**Severity.** Code reviews hunt for P0 and P1 first and rank findings by severity:

- **P0**: takes down shards or the bot, or lets a user abuse it: a panic or hang reachable from
  `/roll`, a startup failure in any sharding mode, mass pings, `/purge` acting without
  permission.
- **P1**: a silently wrong result, or a broken documented roll, on input that passes the
  real-play check.
- **P2**: a minor bug on input that passes the real-play check: a misleading message, wrong
  display or stats, a wrong result only in a rare corner of a real roll.
- **P3**: inputs that fail the real-play check, cleanup, docs and style. A failing input that
  panics, hangs or overflows is still P0.
- List P2 and P3 briefly; don't fix either without asking.

## Adding a Game System

In order, no skipped steps: `aliases.rs` → `Modifier` variant in `dice/mod.rs` → `parser.rs`
(`split_combined_modifiers` and `parse_modifiers_from_part`) → `roller.rs` → `commands/roll.rs` →
`tests/game_systems_tests.rs` → `roll_syntax.md`.

**Do not start by copying the nearest handler** — that is how this codebase accumulated 60
JSCPD clones. Build from the shared helpers and write only what is specific to the new system:

- `RollResult::from_dice(&dice)`, `push_dice_group` — never hand-write those struct literals
- `apply_arithmetic_modifiers`, `is_arithmetic_modifier`, `ArithmeticOp::from_modifier` — `+ - * /`
- `apply_dice_operand`, `apply_modifier_expression` — `+2d6`-style dice operands
- `roll_exploding_die` — reroll while the die shows its maximum
- `indexed_rolls` + `partition_kept_dice` — keep/drop bookkeeping
- `finalize_d10_explosion` — d10 crit/fumble tails (Cyberpunk Red, Witcher)
- `build_roll_set`, `try_parse_roll_set` — roll sets and their parsing
- `checked_add(..).ok_or_else(result_too_large)?` — any arithmetic on `result.total` after the
  user's math is in it; a plain `+=` wraps in release (`1d6 + 2147483646 ess3`)

`CONTRIBUTING.md` § Shared helpers is the canonical list; update it if a helper changes.
Behavioral variants belong in a parameter, never a twin function (see `RerollDirection`,
`MathModifierRules`) — and document *why* the paths differ.

## Testing

- `tests/snapshot_tests.rs` pins observable behavior for every expression in
  `tests/corpus/expressions.txt`. **When you add or change syntax, add corpus lines**: the
  happy path, the boundary values of every numeric parameter, and a `d1` form so the roll is
  pinned deterministically. (A `k1` bug once slipped through because the corpus only had `k2`.)
- When a change goes through a set of modifiers (a predicate, a list of match arms), add a
  corpus line for **every member**, not only the one you were fixing. The 1.8.2 `f#` bug hid
  behind corpus lines that only used `t#`.
- What each snapshot in `tests/snapshots/` pins: `parse.snap` — parsed form and every parse
  error, exactly; `deterministic_rolls.snap` — full result and Discord output of `d1` pools;
  `outcomes.snap` — for random rolls, only result count, success/failure, and error text;
  `seeded_rolls.snap` — exact results at four fixed seeds (covers systems with fixed dice,
  e.g. Savage Worlds, Wrath & Glory).
  `ignored_numbers.snap` lists every expression where an added `+ 0` vs `+ 1000` (leading or
  trailing) changes nothing: known gaps. It should only shrink. A new line is a number your
  change started dropping; fix the change, don't regenerate. A removed line is a fixed gap
  *or* a probe that no longer rolls at all — check which.
- After regenerating snapshots, **read the diff** — every changed line is a change a user
  will see. Never regenerate just to turn a red test green.
- For any parser or roller change, run `tools/diff-vs-release.sh HEAD` and account for every
  changed shape in the commit message; it exits 1 when a roll that worked now differs or
  errors. It rolls the corpus plus generated variants (math
  around every line, a grid of modifier families) at fixed seeds in both trees. Before a
  release, run it against the last `v*` tag, and tag the release (`git tag -a vX.Y.Z`) when it
  ships.
- Run the fuzz test before a release and after adding or changing a game-system handler
  (`FUZZ_ITERS=200000` sweeps deeper). It mutates corpus lines and reports panics (debug builds
  trap the integer overflow release would wrap), hangs, and replies over the limit, each with a
  seed that reproduces it. It stays on demand because a corpus edit reshuffles what it
  generates. Fix each finding and add it to the corpus, which remains the regression guard.
- Signal: if only `seeded_rolls.snap` moved, RNG draw order changed but behavior did not; if
  `deterministic_rolls.snap` moved too, behavior changed.
- Tests are **table-driven** (`vec![(input, expected), ...]`). The table changes; the loop body
  does not. Use and extend the HELPER FUNCTIONS section at the top of `game_systems_tests.rs`
  instead of pasting assertion loops.
- New syntax needs cases for: happy path, common modifiers, comments (`! text`), roll sets,
  and boundary values. Write tests alongside the implementation.
- Game-system mechanics are tested in `game_systems_tests.rs` **only**. Repeating them in
  another test file creates a cross-file clone that no local helper can fix.

## Rust Rules

- Fallible functions return `anyhow::Result<T>`; propagate with `?`; avoid deep nesting.
- No `unwrap()`/`expect()` in production paths, no `panic!()` outside tests, no
  `todo!()`/`unimplemented!()`, no `println!()` (use `tracing`).
- A `_` arm on `Modifier` only where it can't silently ignore one: in matches that pick out
  specific variants (`ArithmeticOp::from_modifier`, `net_advantage_level`), or an arm that
  errors. The roll-pipeline dispatchers in `roller.rs` skip other stages' modifiers with `_`;
  a new modifier with no arm there is caught by its own `game_systems_tests.rs` tests, so
  never skip them.
- SQL uses prepared statements only. Randomness goes through `rng.rs` only.
- Prefer borrowing; justify every `.clone()`. Meaningful names; delete replaced code; no
  versioned names (`process_v2`).
- **Never call `Regex::new` inside a function.** Hoist into `static NAME: Lazy<Regex>` (or
  `Lazy<Vec<Regex>>`). Per-call compilation once made one roll cost 165ms of CPU and dropped
  every shard in the process. `tests/performance_tests.rs` enforces this; add any new
  regex-bearing file to its list.

## Operational Constraints

- `main.rs` startup must keep all three sharding modes working: single process
  (`SHARD_COUNT`), autosharding (`USE_AUTOSHARDING`), and multi-process
  (`SHARD_START` + `TOTAL_SHARDS`).
- Output from `commands/roll.rs` must fit Discord's message limit even for large roll sets;
  truncate with a notice rather than fail the send.
- `/purge` checks permissions before acting — never weaken or bypass this.

## Docs to Keep in Sync

- `roll_syntax.md` — any syntax change
- `README.md` and `env.example` — env vars, commands, deployment
- `CONTRIBUTING.md` — mirror any rule changed here
