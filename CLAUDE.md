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
  their single-character counterparts (`e`, `r`, `k`, `t`). Preserve this ordering.
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

## Testing

| File | Purpose |
| --- | --- |
| `tests/unit_tests.rs` | Core dice logic, parsing, rolling |
| `tests/game_systems_tests.rs` | All game system behavior (consolidated) |
| `tests/integration_tests.rs` | End-to-end functionality |
| `tests/performance_tests.rs` | Performance and roll-limit testing |

- Use **table-driven tests** — `vec![(input, expected), ...]` loops are the established
  pattern; follow it rather than writing one function per case
- Write tests before or alongside the implementation, not after
- New syntax needs cases for: the happy path, combination with common modifiers, comments
  (`! text`), roll sets, and boundary/limit values

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
