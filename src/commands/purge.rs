//! `/purge` slash-command handler.
//!
//! Deletes 2–100 recent messages from the current channel. Requires the
//! **Manage Messages** or **Administrator** permission, enforced both via
//! `default_member_permissions` and at runtime, and checks the bot's own
//! channel permissions before trying. Falls back to sequential single-message
//! deletion if Discord's bulk-delete endpoint fails.

use anyhow::Result;
use serenity::{
    all::{
        CommandDataOptionValue, CommandInteraction, CommandOptionType, GetMessages,
        InteractionContext, MessageId,
    },
    builder::{CreateCommand, CreateCommandOption},
    model::prelude::Permissions,
    prelude::Context,
};
use tracing::warn;

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
        // Not offered in DMs, where there is nothing to purge
        .contexts(vec![InteractionContext::Guild])
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> Result<String> {
    // Check if user has manage messages permission
    // A member is only present in a server. Replying, rather than an Err,
    // shows the user this message instead of a generic failure.
    let Some(member) = command.member.as_ref() else {
        return Ok("❌ This command can only be used in servers.".to_string());
    };
    let has_permission = member
        .permissions
        .is_some_and(|perms| perms.administrator() || perms.manage_messages());

    if !has_permission {
        return Ok(
            "❌ You need the 'Manage Messages' or 'Administrator' permission to use this command."
                .to_string(),
        );
    }

    let Some(count) = purge_count(command.data.options.first().map(|opt| &opt.value)) else {
        return Ok("❌ Message count must be between 2 and 100.".to_string());
    };

    if !bot_can_purge(command.app_permissions) {
        return Ok(
            "❌ I need the 'Manage Messages' and 'Read Message History' permissions in this channel to purge it."
                .to_string(),
        );
    }

    // Get the messages sent before this command. The deferred "thinking"
    // response is newer than the command, so it is never one of them.
    let before_command = MessageId::new(command.id.get());
    let messages = command
        .channel_id
        .messages(
            &ctx.http,
            GetMessages::new().before(before_command).limit(count),
        )
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
            Err(why) => {
                // Bulk delete refuses the whole batch if any message is over
                // two weeks old; deleting one at a time still removes them.
                warn!("Bulk delete failed, deleting one at a time: {why}");
                let mut deleted = 0;
                let mut last_error = None;
                for message in &messages {
                    match command
                        .channel_id
                        .delete_message(&ctx.http, message.id)
                        .await
                    {
                        Ok(()) => deleted += 1,
                        Err(why) => last_error = Some(why),
                    }
                }
                if let Some(why) = last_error {
                    warn!(
                        "Purge deleted {deleted} of {} messages; last error: {why}",
                        messages.len()
                    );
                }
                deleted
            }
        }
    };

    if deleted_count < messages.len() {
        return Ok(format!(
            "🗑️ Deleted {deleted_count} of {} message(s); Discord refused the rest.",
            messages.len()
        ));
    }
    Ok(format!(
        "🗑️ Successfully deleted {deleted_count} message(s)."
    ))
}

/// The requested message count, if it is 2-100. `try_from`, not `as u8`: a
/// cast wrapped 300 to 44 and turned -50 into 50 before the range check ran.
pub fn purge_count(option: Option<&CommandDataOptionValue>) -> Option<u8> {
    match option? {
        CommandDataOptionValue::Integer(count) => u8::try_from(*count)
            .ok()
            .filter(|count| (2..=100).contains(count)),
        _ => None,
    }
}

/// Whether the bot itself can purge this channel: it must read the history to
/// find the messages and manage messages to delete them. Without either,
/// purge used to send a bulk delete plus up to 100 single deletes that all
/// failed, then report "Successfully deleted 0". Discord sends the bot's
/// channel permissions with every interaction; if they are absent, try anyway.
pub fn bot_can_purge(app_permissions: Option<Permissions>) -> bool {
    app_permissions.is_none_or(|perms| {
        perms.administrator() || (perms.manage_messages() && perms.read_message_history())
    })
}
