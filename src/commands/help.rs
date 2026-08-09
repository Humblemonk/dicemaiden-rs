//! `/help` slash-command handler.
//!
//! Returns an ephemeral (visible only to the invoking user) help message.
//! An optional `topic` string-choice selects which section to display.
//!
//! The available topics come from `help_text::HELP_TOPICS` — both the
//! slash-command choices and the dispatch below are derived from it, and
//! `/roll help <topic>` resolves through the same `generate_topic_help`, so
//! there is no per-surface topic list that can drift.  An unrecognised topic
//! falls back to the basic page.

use crate::commands::CommandResponse; // Import CommandResponse
use crate::help_text; // Import the shared help text module from src root
use anyhow::Result;
use serenity::{
    all::{CommandDataOptionValue, CommandInteraction, CommandOptionType},
    builder::{CreateCommand, CreateCommandOption},
    prelude::Context,
};

pub fn register() -> CreateCommand {
    let topic_option = help_text::HELP_TOPICS.iter().fold(
        CreateCommandOption::new(
            CommandOptionType::String,
            "topic",
            "Help topic (basic, alias, system, a5e)",
        )
        .required(false),
        |option, topic| option.add_string_choice(*topic, *topic),
    );

    CreateCommand::new("help")
        .description("Show help information for Dice Maiden")
        .add_option(topic_option)
}

pub async fn run(_ctx: &Context, command: &CommandInteraction) -> Result<CommandResponse> {
    let topic = command
        .data
        .options
        .first()
        .and_then(|opt| match &opt.value {
            CommandDataOptionValue::String(s) => Some(s.as_str()),
            _ => None,
        })
        .unwrap_or("basic");

    // Unknown topics fall back to the basic page rather than erroring
    let help_text =
        help_text::generate_topic_help(topic).unwrap_or_else(help_text::generate_basic_help);

    // Return as private response
    Ok(CommandResponse::private(help_text))
}
