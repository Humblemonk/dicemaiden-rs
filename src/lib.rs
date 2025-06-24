// src/lib.rs
// This file exposes the library API for testing and external use

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
    parse_and_roll, 
    format_multiple_results, 
    format_multiple_results_with_limit,
    DiceRoll, 
    Modifier, 
    RollResult,
    HeroSystemType,
    DiceGroup,
};

// Re-export dice submodules for testing
pub use dice::aliases;
pub use dice::parser;
pub use dice::roller;
