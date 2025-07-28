use crate::commands::CommandResponse; // Import CommandResponse
use crate::help_text; // Import the shared help text module from src root
use anyhow::Result;
use serenity::{
    all::{CommandDataOptionValue, CommandInteraction, CommandOptionType},
    builder::{CreateCommand, CreateCommandOption},
    prelude::Context,
};

pub fn register() -> CreateCommand {
    CreateCommand::new("help")
        .description("Show help information for Dice Maiden")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "topic",
                "Help topic (basic, alias, system, a5e)",
            )
            .required(false)
            .add_string_choice("basic", "basic")
            .add_string_choice("alias", "alias")
            .add_string_choice("system", "system")
            .add_string_choice("a5e", "a5e")
            .add_string_choice("aliens", "aliens"),
        )
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

    let help_text = match topic {
        "alias" => help_text::generate_alias_help(),
        "system" => help_text::generate_system_help(),
        "a5e" => help_text::generate_a5e_help(),
        "aliens" => help_text::generate_aliens_help(),
        _ => help_text::generate_basic_help(),
    };

    // Return as private response
    Ok(CommandResponse::private(help_text))
}
