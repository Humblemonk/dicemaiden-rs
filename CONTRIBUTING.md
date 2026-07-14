# Contributing to Dice Maiden

Thanks for your interest in contributing! Dice Maiden is an open-source Discord dice rolling bot written in Rust. This guide covers everything you need to get started.

---

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Adding a New Game System](#adding-a-new-game-system)
- [Testing](#testing)
- [Code Standards](#code-standards)
- [Submitting a Pull Request](#submitting-a-pull-request)

---

## Getting Started

1. **Fork** the repository on GitHub
2. **Clone** your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/dicemaiden-rs.git
   cd dicemaiden-rs
   ```
3. **Create a feature branch**:
   ```bash
   git checkout -b my-feature
   ```

### Requirements

- Rust 1.90+ (see `rust-version` in `Cargo.toml`)
- A Discord bot token (for manual testing; not needed to run the test suite)
- SQLite (the bot creates the database automatically)

---

## Development Setup

```bash
# Copy the example environment file
cp env.example .env
# Edit .env and add your Discord bot token if you want to run the bot locally

# Build
cargo build

# Run all tests (no token required)
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

When running the bot locally for manual testing, set `GUILD_ID` in your `.env` to your test server's ID. This registers slash commands instantly to that guild rather than waiting up to an hour for global propagation.

---

## Making Changes

### Before you write any code

**Dice syntax is a public API.** This bot runs on hundreds of thousands of servers, and a silently changed roll result is the worst possible bug. Never change the behavior of an existing expression unless the change is explicitly requested. If your change *might* alter existing behavior and you're unsure, raise the compatibility question in your PR or an issue instead of deciding yourself.

Read [`roll_syntax.md`](roll_syntax.md) to understand the existing dice syntax. The parser uses **prefix matching**, so new syntax identifiers must not overlap with — or be a prefix of — any existing ones. Conflicts are a real risk and can silently break unrelated rolls.

### Data flow

```
src/dice/parser.rs → src/dice/roller.rs → src/commands/roll.rs → Discord message
```

- **`src/dice/aliases.rs`** — expands game system shorthand into standard expressions. Alias expansion happens **before** parsing; its output is an ordinary expression string.
- **`src/dice/parser.rs`** — turns a dice expression string into `Vec<DiceRoll>`
- **`src/dice/roller.rs`** — executes rolls, applies modifiers, returns `Vec<RollResult>`
- **`src/commands/roll.rs`** — formats results into a Discord message string

### Display formatting constraints

Discord enforces a hard message length cap (2000 characters). If you touch formatting in `src/commands/roll.rs`, output must stay within the cap even for large roll sets — prefer truncation with a notice over a failed send. Existing formatting code shows the established fallback pattern.

### Mandatory quality check

Run this before every commit and before opening a PR. All three must pass — no exceptions:

```bash
cargo clippy -- -D warnings && cargo fmt --check && cargo test
```

---

## Adding a New Game System

Follow this sequence exactly — do not skip or reorder steps:

1. **`src/dice/aliases.rs`** — add the alias expansion function
2. **`src/dice/mod.rs`** — add the `Modifier` enum variant
3. **`src/dice/parser.rs`** — add parsing logic in `split_combined_modifiers` and the modifier parser
4. **`src/dice/roller.rs`** — add roll execution logic
5. **`src/commands/roll.rs`** — add display/formatting logic
6. **`tests/game_systems_tests.rs`** — add tests following existing patterns
7. **`roll_syntax.md`** — document the new syntax

### Modifier ordering rules

- **Drop before explode** — dropped dice are never reconsidered for explosion. This is intentional and tested. Do not change the ordering.
- Multi-character prefixes (`ie`, `irg`, `ir`, `km`, `kl`, `tl`) must be matched **before** their single-character counterparts (`e`, `r`, `k`, `t`).

---

## Testing

Tests live in `tests/`:

| File | Purpose |
|---|---|
| `unit_tests.rs` | Core dice logic, parsing, rolling |
| `game_systems_tests.rs` | All game system tests |
| `integration_tests.rs` | End-to-end functionality |
| `performance_tests.rs` | Performance and limit testing |

Use **table-driven tests** — this is the established pattern in the project:

```rust
let cases = vec![
    ("4d6k3", expected_output_a),
    ("2d10e ! Fire damage", expected_output_b),
];
for (input, expected) in cases {
    // assert...
}
```

Write tests **before** or **alongside** your implementation — not after.

### What sufficient coverage looks like

New syntax or game systems need cases for **all** of the following:

- The happy path
- Combination with common modifiers
- Comments (`! text`)
- Roll sets
- Boundary and limit values

**Bug fixes must include a regression test** reproducing the original bug, added alongside the fix.

---

## Code Standards

### Required

- Return `anyhow::Result<T>` from all fallible functions
- Use `?` for error propagation — avoid deep `match`/`if let` nesting
- Use `tracing::{info!, warn!, error!, debug!}` for all logging
- All randomness goes through `src/dice/rng.rs` — never instantiate ad-hoc RNGs elsewhere

### Forbidden

| Forbidden | Use instead |
|---|---|
| `unwrap()` / `expect()` in production paths | `?` and `anyhow::Result` |
| `panic!()` outside tests | Propagate the error |
| `println!()` | `tracing::info!` |
| `todo!()` / `unimplemented!()` in final code | Implement it or leave it out |
| String concatenation in SQL | Prepared statements only |
| Ad-hoc RNGs (`rand::rng()`, etc.) | `src/dice/rng.rs` |
| Unnecessary `.clone()` | Prefer borrowing |

### Style

- Meaningful names: `dice_count` not `n`, `shard_id` not `id`
- Exhaustive match arms — avoid wildcard `_` that silently swallows variants (especially on the `Modifier` enum, where a missed arm means a modifier is silently ignored)
- Delete replaced code — do not leave old and new implementations side by side
- No versioned function names (`process_v2`, `handle_new`)

### Docs to keep in sync

- `roll_syntax.md` — update with any syntax change
- `env.example` — update when adding or changing an environment variable
- `README.md` — update if your change affects env vars, commands, or deployment

---

## Submitting a Pull Request

1. Ensure `cargo clippy -- -D warnings && cargo fmt --check && cargo test` all pass
2. If you added a game system, confirm `roll_syntax.md` is updated
3. Keep your PR focused — one feature or fix per PR
4. Write a clear description of what you changed and why
5. Reference any related issues (e.g., `Closes #123`)

PRs that fail the quality check or lack tests for new functionality will be asked to revise before review.

---

## Questions?

- Open an issue for bugs or feature requests
- Join the [Discord community](https://discord.gg/AYNcxc9NeU) for discussion
- Check [`roll_syntax.md`](roll_syntax.md) for the full dice syntax reference
