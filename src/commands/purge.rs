use anyhow::Result;
use serenity::{
    all::{CommandDataOptionValue, CommandInteraction, CommandOptionType, GetMessages},
    builder::{CreateCommand, CreateCommandOption},
    model::prelude::Permissions,
    prelude::Context,
};

pub fn register() -> CreateCommand {
    CreateCommand::new("purge")
        .description("Purge recent messages (requires manage messages permission)")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Integer,
                "count",
                "Number of messages to delete (2-100)",
            )
            .required(true)
            .min_int_value(2)
            .max_int_value(100),
        )
        .default_member_permissions(Permissions::MANAGE_MESSAGES)
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> Result<String> {
    // Check if user has manage messages permission
    let has_permission = match command.member.as_ref() {
        Some(member) => {
            let _guild_id = command
                .guild_id
                .ok_or_else(|| anyhow::anyhow!("Purge command can only be used in servers"))?;

            // Check if user has admin or manage messages permission
            member
                .permissions
                .is_some_and(|perms| perms.administrator() || perms.manage_messages())
        }
        None => return Err(anyhow::anyhow!("This command can only be used in servers")),
    };

    if !has_permission {
        return Ok(
            "❌ You need the 'Manage Messages' or 'Administrator' permission to use this command."
                .to_string(),
        );
    }

    let count = command
        .data
        .options
        .first()
        .and_then(|opt| match &opt.value {
            CommandDataOptionValue::Integer(i) => Some(i.unsigned_abs() as u8),
            _ => None,
        })
        .unwrap_or(10);

    if !(2..=100).contains(&count) {
        return Ok("❌ Message count must be between 2 and 100.".to_string());
    }

    // Get recent messages
    let messages = command
        .channel_id
        .messages(&ctx.http, GetMessages::new().limit(count))
        .await?;

    if messages.is_empty() {
        return Ok("❌ No messages found to delete.".to_string());
    }

    // Delete messages
    let deleted_count = if messages.len() == 1 {
        // Delete single message
        command
            .channel_id
            .delete_message(&ctx.http, messages[0].id)
            .await?;
        1
    } else {
        // Bulk delete (Discord API limitation: messages must be less than 2 weeks old)
        let message_ids: Vec<_> = messages.iter().map(|m| m.id).collect();

        match command
            .channel_id
            .delete_messages(&ctx.http, &message_ids)
            .await
        {
            Ok(_) => message_ids.len(),
            Err(_) => {
                // Fallback to individual deletion if bulk delete fails
                let mut deleted = 0;
                for message in &messages {
                    if command
                        .channel_id
                        .delete_message(&ctx.http, message.id)
                        .await
                        .is_ok()
                    {
                        deleted += 1;
                    }
                }
                deleted
            }
        }
    };

    Ok(format!(
        "🗑️ Successfully deleted {} message(s).",
        deleted_count
    ))
}
