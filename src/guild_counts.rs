//! Server and member counts for `bot-info` and the statistics task.
//!
//! Replaces Serenity's guild cache, which stored every server's channels, roles
//! and emojis (~50 KB per server) only for these two numbers to be read.
//!
//! Counting rules mirror Serenity's cache so the reported totals do not shift:
//! servers listed in `Ready` and servers that go temporarily unavailable still
//! count; only a delete with `unavailable: false` (the bot left) removes one.
//! One deliberate difference: an unavailable server keeps its member count
//! instead of dropping to 0, so a Discord outage no longer dips the user total.
//! Member counts come from `GUILD_CREATE` only, as before — the bot does not
//! request the `GUILD_MEMBERS` intent, so Serenity never updated them either.

use serenity::model::id::GuildId;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

#[derive(Default)]
pub struct GuildCounts {
    member_counts: Mutex<HashMap<GuildId, u64>>,
}

impl GuildCounts {
    fn lock(&self) -> MutexGuard<'_, HashMap<GuildId, u64>> {
        self.member_counts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Servers listed in `Ready` count before their `GUILD_CREATE` arrives.
    /// `or_insert` because Serenity runs each event handler as its own task, so a
    /// shard's `GUILD_CREATE` handler can run before its `Ready` handler.
    pub fn ready(&self, guild_ids: impl IntoIterator<Item = GuildId>) {
        let mut counts = self.lock();
        for guild_id in guild_ids {
            counts.entry(guild_id).or_insert(0);
        }
    }

    pub fn create(&self, guild_id: GuildId, member_count: u64) {
        self.lock().insert(guild_id, member_count);
    }

    pub fn delete(&self, guild_id: GuildId, unavailable: bool) {
        let mut counts = self.lock();
        if unavailable {
            counts.entry(guild_id).or_insert(0);
        } else {
            counts.remove(&guild_id);
        }
    }

    /// `(servers, members)`
    pub fn totals(&self) -> (usize, usize) {
        let counts = self.lock();
        let members = counts.values().map(|&count| count as usize).sum();
        (counts.len(), members)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    enum Event {
        Ready(&'static [u64]),
        Create(u64, u64),
        Delete(u64, bool),
    }
    use Event::*;

    #[test]
    fn totals_follow_gateway_events() {
        let cases: Vec<(&str, Vec<Event>, (usize, usize))> = vec![
            (
                "ready counts servers before create",
                vec![Ready(&[1, 2])],
                (2, 0),
            ),
            (
                "create fills member count",
                vec![Ready(&[1, 2]), Create(1, 10)],
                (2, 10),
            ),
            (
                "create before ready keeps its count",
                vec![Create(1, 10), Ready(&[1, 2])],
                (2, 10),
            ),
            (
                "create replaces old count",
                vec![Create(1, 10), Create(1, 12)],
                (1, 12),
            ),
            (
                "unavailable server still counts",
                vec![Create(1, 10), Delete(1, true)],
                (1, 10),
            ),
            (
                "unknown unavailable server counts",
                vec![Delete(3, true)],
                (1, 0),
            ),
            (
                "leaving a server removes it",
                vec![Create(1, 10), Create(2, 5), Delete(1, false)],
                (1, 5),
            ),
        ];

        for (name, events, expected) in cases {
            let counts = GuildCounts::default();
            for event in events {
                match event {
                    Ready(ids) => counts.ready(ids.iter().map(|&id| GuildId::new(id))),
                    Create(id, members) => counts.create(GuildId::new(id), members),
                    Delete(id, unavailable) => counts.delete(GuildId::new(id), unavailable),
                }
            }
            assert_eq!(counts.totals(), expected, "{name}");
        }
    }
}
