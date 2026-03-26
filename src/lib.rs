//! Dice Maiden library crate — public API surface shared between the binary and
//! the integration / game-system test suites.
//!
//! The bot binary (`main.rs`) is built on top of this crate.  Splitting the
//! code into a library crate lets the test suites import internal modules
//! directly without having to go through Discord.
//!
//! # Module layout
//!
//! ```text
//! dicemaiden_rs
//! ├── commands/        Discord slash-command handlers (roll, help, purge)
//! ├── database         SQLite statistics persistence
//! ├── dice/            Core dice engine
//! │   ├── mod.rs       Types: DiceRoll, RollResult, Modifier, DiceGroup
//! │   ├── aliases.rs   Game-system alias expansion
//! │   ├── parser.rs    Text → Vec<DiceRoll>
//! │   ├── roller.rs    Vec<DiceRoll> → Vec<RollResult>
//! │   ├── roll.rs      RollResult → Discord message string
//! │   └── rng.rs       Enhanced RNG seeding
//! └── help_text.rs     Static help message generators
//! ```
//!
//! # Re-exports
//!
//! The most-used types (`DiceRoll`, `RollResult`, `Modifier`, `DiceGroup`,
//! `parse_and_roll`, `format_multiple_results`) are re-exported from the crate
//! root for convenience in tests and external consumers.
//!
//! [`ShardManagerContainer`] and [`DatabaseContainer`] are Serenity
//! [`TypeMapKey`] wrappers that allow the shard manager and database handles to
//! be stored in, and retrieved from, the Serenity shared data map.

pub mod commands;
pub mod database;
pub mod dice;
pub mod help_text;

use serenity::prelude::*;
use std::sync::Arc;

// Move these type map keys from main.rs to lib.rs so they can be shared
pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<serenity::gateway::ShardManager>;
}

pub struct DatabaseContainer;

impl TypeMapKey for DatabaseContainer {
    type Value = Arc<database::Database>;
}

// Re-export commonly used items for easier testing
pub use dice::{
    DiceGroup, DiceRoll, HeroSystemType, Modifier, RollResult, format_multiple_results,
    format_multiple_results_with_limit, parse_and_roll,
};

// Re-export dice submodules for testing
pub use dice::aliases;
pub use dice::parser;
pub use dice::roller;
