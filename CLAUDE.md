# dicemaiden-rs

Discord dice rolling bot in Rust (Serenity framework). Complete rewrite of the original
Ruby DiceMaiden, running in production on 500,000+ Discord servers — correctness and
backward compatibility of dice syntax outweigh cleverness or new abstractions. A silently
changed roll result is the worst possible bug.

Single binary, SQLite for shard statistics, slash commands: `/roll`, `/r`, `/help`, `/purge`.

## Verification

Before considering any change complete — all three must pass, no exceptions:

```
cargo clippy -- -D warnings && cargo fmt --check && cargo test
```

Tests run without a Discord token. For manual testing against a live guild, set `GUILD_ID`
in `.env` so slash commands register instantly instead of waiting for global propagation.

CI additionally runs super-linter, whose JSCPD copy-paste check has a threshold of **0** —
any clone of 5+ lines / 50+ tokens fails the build. Run the same check locally before
pushing (same config CI uses):

```
npx jscpd@3 --config .github/linters/.jscpd.json src tests
```

## Architecture

```
src/
  main.rs         # Entry point, Discord client setup, sharding
  database.rs     # SQLite (shard statistics) — prepared statements only
  help_text.rs    # Shared help text for all help commands
  lib.rs          # Exposes internals for the test suite
  dice/
    mod.rs        # Core types: DiceRoll, RollResult, Modifier enum
    parser.rs     # Expression string → Vec<DiceRoll>; syntax validation
    roller.rs     # Roll execution and modifier application → Vec<RollResult>
    rng.rs        # Cryptographically secure RNG, multiple entropy sources
    aliases.rs    # Game system shorthand → standard expression expansion
  commands/
    mod.rs        # Command exports, CommandResponse type
    roll.rs       # Roll command + result formatting for Discord
    help.rs       # Topic-based help
    purge.rs      # Message purge with permission checking
```

**Data flow:** `parser.rs` → `roller.rs` → `commands/roll.rs` → Discord message.
Aliases expand *before* parsing (`aliases.rs` output is a standard expression string).

The Ruby artifacts in the repository root (`Gemfile`, `.rubocop*.yml`) relate to the original
Ruby bot's history/tooling — they are not part of the Rust build. Don't modify them.

## Parser Invariants (critical)

- The parser uses **prefix matching**. A new syntax identifier that overlaps with or is a
  prefix of an existing one can silently break unrelated rolls. Check `roll_syntax.md` for
  every existing token before introducing new syntax.
- Multi-character prefixes (`ie`, `irg`, `ir`, `km`, `kl`, `tl`) must be matched **before**
  their single-character counterparts (`e`, `r`, `k`, `t`). Preserve this ordering. The
  ordered pattern lists live in the `COMBINED_MODIFIER_PATTERNS`, `SPLIT_MODIFIER_PATTERNS`,
  and `MODIFIER_START_PATTERNS` statics in `parser.rs` — they are `Vec<Regex>` rather than
  sets precisely because order is load-bearing. Never sort or dedupe them.
- **Drop before explode** — dropped dice are never reconsidered for explosion. This is
  intentional and covered by tests. Do not change modifier ordering semantics.
- Dice syntax is a public API. Never change the behavior of an existing expression without
  being explicitly asked; when in doubt, flag the compatibility question instead of deciding.

## Adding a New Game System

Follow this sequence, no skipped or reordered steps:

1. `src/dice/aliases.rs` — alias expansion function
2. `src/dice/mod.rs` — `Modifier` enum variant
3. `src/dice/parser.rs` — parsing logic in `split_combined_modifiers` and the modifier parser
4. `src/dice/roller.rs` — roll execution logic
5. `src/commands/roll.rs` — display/formatting logic
6. `tests/game_systems_tests.rs` — tests following existing patterns
7. `roll_syntax.md` — document the new syntax

**Do not start by copy-pasting the nearest existing handler.** That is how this codebase
accumulated 60 JSCPD clones. Start from the shared helpers below and write only the part
that is actually specific to the new system.

## Shared Helpers (use these instead of copy-pasting)

Every one of these exists because the same block had been pasted 2–10 times. If you are
about to write something that resembles a row on the left, the right-hand column already
does it.

| Instead of writing | Call |
| --- | --- |
| The 30-field `RollResult { .. }` literal | `RollResult::from_dice(&dice)` — or `RollResult { field: x, ..RollResult::from_dice(&dice) }` when a field differs |
| A loop applying `+ - * /` to `result.total` | `apply_arithmetic_modifiers(&dice.modifiers, &mut result.total)` |
| `matches!(m, Modifier::Add(_) \| Modifier::Subtract(_) \| ...)` | `is_arithmetic_modifier(m)`, or `ArithmeticOp::from_modifier(m)?` when the operand is needed |
| Rolling `+2d6`-style operands into an expression | `apply_dice_operand(..)` / `apply_modifier_expression(..)` |
| A roll-and-explode-while-max loop | `roll_exploding_die(rng, sides)` |
| Keep/drop index bookkeeping | `indexed_rolls(result)` + `partition_kept_dice(result, &kept)` |
| A d10 crit/fumble explosion tail (CPR, Witcher) | `finalize_d10_explosion(..)` |
| Building `Set 1..N` copies, or set parsing in `parser.rs` | `build_roll_set(..)` / `try_parse_roll_set(..)` |

Behavioral variants belong in a parameter, not a second copy of the function — see
`RerollDirection` (`r` vs `rg`) and `MathModifierRules` (standard vs post-division), each
of which replaced a near-identical twin function. When you do add a parameter like this,
document *why* the two paths differ; that difference is usually load-bearing.

## Testing

| File | Purpose |
| --- | --- |
| `tests/unit_tests.rs` | Core dice logic, parsing, rolling |
| `tests/game_systems_tests.rs` | All game system behavior (consolidated) |
| `tests/integration_tests.rs` | End-to-end functionality |
| `tests/performance_tests.rs` | Performance and roll-limit testing |
| `tests/snapshot_tests.rs` | Golden snapshots pinning observable behavior |

### Snapshot tests — the regression net

`tests/snapshot_tests.rs` pins what users can observe for every expression in
`tests/corpus/expressions.txt`, so that a refactor which silently changes a roll fails
`cargo test` instead of shipping:

- **`parse.txt`** — parsing is deterministic, so the parsed form and every parse error are
  pinned exactly, for the whole corpus.
- **`deterministic_rolls.txt`** — expressions whose dice cannot vary (`d1` pools) have their
  full result *and* their formatted Discord output pinned exactly.
- **`outcomes.txt`** — for randomly-rolling expressions, only what cannot vary with the dice:
  result count, success/failure of the call, and exact error text.

**When you add or change syntax, add expressions to `tests/corpus/expressions.txt`** — a
happy-path case, the boundary values of every numeric parameter, and a `d1` form so the roll
lands in the deterministic snapshot. Coverage here is only as good as the corpus: a keep-count
bug at `k1` slipped through when the corpus had `k2` but not `k1`.

Regenerate after an intentional change, then **read the diff** — every changed line is a
change a user will see:

```
UPDATE_SNAPSHOTS=1 cargo test --test snapshot_tests
```

Never regenerate to make a red test go green without reading what moved.

What snapshots cannot cover: results of randomly-rolling systems. `roll_dice` builds its own
RNG internally (`rng.rs`), so those cannot be reproduced exactly. Cover them with the
targeted assertions in `game_systems_tests.rs`.

- Use **table-driven tests** — `vec![(input, expected), ...]` loops are the established
  pattern; follow it rather than writing one function per case
- Write tests before or alongside the implementation, not after
- New syntax needs cases for: the happy path, combination with common modifiers, comments
  (`! text`), roll sets, and boundary/limit values
- **The table changes; the loop body does not.** A pasted assertion loop is the most common
  source of JSCPD clones here. `tests/game_systems_tests.rs` has a HELPER FUNCTIONS section
  at the top — use it, and add to it rather than pasting:
  `roll_one`, `assert_labelled_roll_sets`, `assert_alias_matches_expansion`,
  `assert_success_alias`, `assert_no_prefix_conflicts`, `assert_kept_and_dropped`,
  `assert_valid`, `assert_invalid`
- Keep each file to its own purpose. A game system's mechanics are tested in
  `game_systems_tests.rs` **only** — re-testing the same scenarios in `integration_tests.rs`
  is a cross-file clone, and cross-file clones cannot be fixed with a local helper
  (each test file is its own crate)

## Rust Rules

- All fallible functions return `anyhow::Result<T>`; propagate with `?`, avoid deep
  `match`/`if let` nesting
- No `unwrap()`/`expect()` in production paths; no `panic!()` outside tests; no
  `todo!()`/`unimplemented!()` in final code
- No `println!()` — use `tracing::{info!, warn!, error!, debug!}`
- SQL uses prepared statements only — never string concatenation
- Prefer borrowing; justify every `.clone()`
- Exhaustive match arms — avoid wildcard `_` that silently swallows variants (especially
  on the `Modifier` enum, where a missed arm means a modifier is silently ignored)
- Meaningful names (`dice_count` not `n`); delete replaced code; no versioned function
  names (`process_v2`, `handle_new`)
- Randomness goes through `rng.rs` only — never instantiate ad-hoc RNGs elsewhere
- **Never call `Regex::new` inside a function.** Hoist every pattern into a
  `static NAME: Lazy<Regex>` (or `Lazy<Vec<Regex>>` for an ordered list). Compiling a regular expression
  costs ~35µs against ~1µs to match with it, and the parser evaluates ~20 patterns per input
  token — per-call compilation once made a single legal roll cost 165ms of CPU and starved
  the gateway until every shard in the process dropped. `tests/performance_tests.rs` enforces
  this; add any new regex-bearing source file to the list there.

## Operational Context

- Sharding: single process by default (`SHARD_COUNT`), optional autosharding
  (`USE_AUTOSHARDING`), and multi-process sharding via `SHARD_START` + `TOTAL_SHARDS`.
  Changes to `main.rs` startup must keep all three modes working.
- `env.example` documents all environment variables — update it when adding one.
- Discord message limits are real constraints: output formatting in `commands/roll.rs`
  must stay within Discord's message length caps even for large roll sets; prefer
  truncation with a notice over a failed send.
- `/purge` performs permission checks before acting — never weaken or bypass them.

## Docs to Keep in Sync

- `roll_syntax.md` — user-facing syntax reference; update with any syntax change
- `README.md` — env vars, commands, deployment examples
- `CONTRIBUTING.md` — contributor workflow; if a rule changes here, mirror it there
