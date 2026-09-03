# Dice Maiden — Rust Edition

A Discord dice rolling bot for tabletop RPGs. Supports complex dice expressions, exploding
dice, keep/drop, success counting, and built-in aliases for 30+ game systems.

A complete rewrite of the original [DiceMaiden](https://github.com/Humblemonk/DiceMaiden)
Ruby bot in Rust.

<p align="center">
<a href="https://top.gg/bot/377701707943116800">
    <img src="https://top.gg/api/widget/377701707943116800.svg" alt="Dice Maiden" />
</a>
</p>

## Add to Your Server

[**Click here to add Dice Maiden**](https://discord.com/api/oauth2/authorize?client_id=572301609305112596&permissions=274878000128&scope=bot%20applications.commands)

The bot appears in your default public channel with permission to read, send, and manage
messages.

> **Users need the "Use Application Commands" permission to use slash commands.**

To restrict the bot to specific channels, go to **Server Settings → Integrations → Dice Maiden**.

## Commands

| Command | Description |
| --- | --- |
| `/roll <dice>` | Roll dice using RPG notation |
| `/r <dice>` | Short alias for `/roll` |
| `/help [topic]` | Help — topics: `basic`, `alias`, `system`, `a5e`, `aliens`, `mothership` |
| `/purge <count>` | Delete recent messages (requires Manage Messages) |
| `/roll donate` | Support information |

## Rolling Dice

![Example roll](https://github.com/user-attachments/assets/0371ff72-e3da-4400-9e1b-8063ef8554a7)

```text
/roll 2d6 + 3           # Basic roll with modifier
/roll 4d6 k3            # Keep the 3 highest
/roll 10d6 e6 k8 +4     # Explode 6s, keep 8 highest, add 4
/roll 4d10 t8 ie10 f1   # Success counting with botches
/roll 6 4d6             # Six sets of 4d6
/roll (Fireball) 8d6 ! AOE   # Labeled roll with a comment
```

**[Full syntax reference and game system aliases →](roll_syntax.md)**

## Self-Hosting

Requires Rust 1.95+ and a Discord bot token. SQLite is created automatically.

```bash
git clone https://github.com/Humblemonk/dicemaiden-rs.git
cd dicemaiden-rs
cp env.example .env      # add your DISCORD_TOKEN
cargo build --release
cargo run --release
```

Create your bot at the [Discord Developer Portal](https://discord.com/developers/applications)
and invite it with Send Messages, Use Slash Commands, Manage Messages, and Read Message
History.

### Environment Variables

See [`env.example`](env.example) for the full annotated list.

| Variable | Required | Description |
| --- | --- | --- |
| `DISCORD_TOKEN` | ✓ | Bot token from the Discord developer portal |
| `DATABASE_URL` | | SQLite path — defaults to `./main.db`, created if missing |
| `GUILD_ID` | | Register commands to one guild for instant testing |
| `SHARD_COUNT` | | Shards for this process — defaults to 1 |
| `USE_AUTOSHARDING` | | `true` lets Discord pick the shard count |
| `SHARD_START` | | First shard ID (multi-process sharding) |
| `TOTAL_SHARDS` | | Total shards across all processes (multi-process sharding) |
| `MAX_CONCURRENCY` | | Hint only — Discord overrides with your bot's real limit |
| `RUST_LOG` | | Log level — defaults to `info` |

### Container

A production [`Dockerfile`](Dockerfile) is included — build it directly rather than copying
one out of these docs:

```bash
docker build -t dicemaiden-rs .
docker run --env-file .env -v dicemaiden-data:/app/data dicemaiden-rs
```

<details>
<summary><b>Systemd service</b></summary>

```ini
[Unit]
Description=Dice Maiden
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=dicebot
Group=dicebot
WorkingDirectory=/opt/dicemaiden-rs
Environment=RUST_LOG=info
EnvironmentFile=/opt/dicemaiden-rs/.env
ExecStart=/opt/dicemaiden-rs/target/release/dicemaiden-rs
Restart=always
RestartSec=10
TimeoutStartSec=300
TimeoutStopSec=120

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/dicemaiden-rs/data
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

</details>

<details>
<summary><b>Multi-process sharding</b></summary>

Each process owns a contiguous shard range and shares one SQLite database.

```bash
# 3 processes handling 64 shards total
SHARD_COUNT=21 SHARD_START=0  TOTAL_SHARDS=64 ./dicemaiden-rs &   # shards 0-20
SHARD_COUNT=21 SHARD_START=21 TOTAL_SHARDS=64 ./dicemaiden-rs &   # shards 21-41
SHARD_COUNT=22 SHARD_START=42 TOTAL_SHARDS=64 ./dicemaiden-rs &   # shards 42-63
```

</details>

## Contributing

Setup, code standards, testing patterns, and the process for adding a new game system are in
[`CONTRIBUTING.md`](CONTRIBUTING.md).

## License

GPLv3

## Support

- [Open an issue](https://github.com/Humblemonk/dicemaiden-rs/issues) for bugs or feature requests
- [Join the Discord](https://discord.gg/AYNcxc9NeU) for help and discussion
- [`roll_syntax.md`](roll_syntax.md) for dice syntax questions
