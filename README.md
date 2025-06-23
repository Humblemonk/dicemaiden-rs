# DiceMaiden - Rust Edition

A powerful Discord dice rolling bot written in Rust using the Serenity framework. This is a complete rewrite of the original [DiceMaiden](https://github.com/Humblemonk/DiceMaiden) Ruby bot, featuring all the same functionality with improved performance and memory safety.
<p align="center">
<a href="https://top.gg/bot/377701707943116800">
    <img src="https://top.gg/api/widget/377701707943116800.svg" alt="Dice Maiden" />
</a>

## Features

- **Comprehensive Dice Rolling**: Support for complex dice expressions with modifiers
- **Game System Aliases**: Built-in support for popular RPG systems
- **Slash Commands**: Modern Discord integration with `/roll` and `/r` commands
- **Advanced Modifiers**: Exploding dice, keep/drop, rerolls, success counting, and more
- **Multiple Roll Types**: Single rolls, roll sets, and multi-roll expressions
- **Message Management**: Purge command for cleaning up chat

## Quick Install

Follow the link below to add your bot to your Discord server:

<https://discord.com/api/oauth2/authorize?client_id=572301609305112596&permissions=274878000128&scope=bot%20applications.commands>

This will authorize the bot for your server and you should see it in your default public channel. The bot will have permissions to read, send and manage messages.

**Note:** It is recommended to review your app integration settings found under Server Settings > Integrations. From here you can restrict the bot slash commands to specific channels

## Commands

- `/roll <dice>` - Roll dice using RPG notation
- `/r <dice>` - Short alias for roll
- `/help [topic]` - Show help (topics: basic, alias, system)
- `/purge <count>` - Delete recent messages (requires permissions)

## Dice Rolling Syntax

### Example Dice Roll

![readme](https://github.com/user-attachments/assets/0371ff72-e3da-4400-9e1b-8063ef8554a7)

Supported dice rolls and game systems can be [found here!](roll_syntax.md)

**NOTE: USERS WILL NEED THE ROLE "USE APPLICATION COMMANDS" TO USE SLASH COMMANDS**

## Local Instance Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/Humblemonk/dicemaiden-rs.git
   cd dicemaiden-rs
   ```

2. **Set up environment**
   ```bash
   cp env.example .env
   # Edit .env and add your Discord bot token. review the other ENV variables found in this documentation
   ```

3. **Build and run**
   ```bash
   cargo build --release
   cargo run
   ```

### Discord Bot Setup

1. Go to the [Discord Developer Portal](https://discord.com/developers/applications)
2. Create a new application and bot
3. Copy the bot token to your `.env` file
4. Invite the bot to your server with the following permissions:
   - Send Messages
   - Use Slash Commands
   - Manage Messages (for purge command)
   - Read Message History

**Invite URL Template:**
```text
https://discord.com/api/oauth2/authorize?client_id=YOUR_BOT_ID&permissions=274878000128&scope=bot%20applications.commands
```

## Configuration

### Environment Variables
- `DISCORD_TOKEN` - Your Discord bot token (required)
- `GUILD_ID` - Guild ID for testing commands (optional)
- `DATABASE_URL` - SQLite database path. Defaults to sqlite:main.db . This database is automatically created if it doesn't exist (optional)
- `SHARD_COUNT` - Manual number of shards to use. Defaults to 1 for small bots (optional)
- `USE_AUTOSHARDING` - Set to true to use discord recommended shard count. Defaults to false (optional)
- `MAX_CONCURRENCY` - Max concurrent shard connections. Discord will override this with your bots actual limit (optional)
- `RUST_LOG` - Log level (default: info)
- `SHARD_START` - Starting shard ID for the process (needed for multi-process sharding)
- `TOTAL_SHARDS` - Total shards across all processes (needed for multi-process sharding)

You can customize the build further by modifying `Cargo.toml` dependencies.

## Development


### Requirements
- Rust 1.75+
- Discord bot token
- SQLite database (automatically created for bot statistics)
- Dependencies - For a detailed list, review [Cargo.toml](Cargo.toml)

### Building
```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Project Structure
```text
src/
├── main.rs             # Application entry point and Discord client setup
├── database.rs         # SQLite database management for shard statistics
├── help_text.rs        # Shared help text generation for all help commands
├── dice/
│   ├── mod.rs          # Dice module exports and core types (DiceRoll, RollResult, etc.)
│   ├── parser.rs       # Dice expression parsing and syntax validation
│   ├── roller.rs       # Dice rolling execution and modifier application
│   └── aliases.rs      # Game system aliases and expression expansions
└── commands/
    ├── mod.rs          # Command module exports and CommandResponse type
    ├── roll.rs         # Roll command implementation with system info
    ├── help.rs         # Help command with topic-based help system
    └── purge.rs        # Message purge command with permission checking
```

## Deployment


### Docker
```dockerfile
FROM rust:1.70-slim-bookworm as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifest files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY . .
# Force rebuild of our code but reuse dependencies
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -u 1000 dicebot

COPY --from=builder /app/target/release/dicemaiden-rs /usr/local/bin/

# Create data directory for SQLite database
RUN mkdir -p /data && chown dicebot:dicebot /data

USER dicebot
WORKDIR /data

CMD ["dicemaiden-rs"]
```

### Systemd Service
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
### Multi-Process Sharding
```bash
# Example: 3 processes handling 64 shards total
# Process 1: shards 0-20
SHARD_COUNT=21 SHARD_START=0 TOTAL_SHARDS=64 ./dicemaiden-rs &

# Process 2: shards 21-41  
SHARD_COUNT=21 SHARD_START=21 TOTAL_SHARDS=64 ./dicemaiden-rs &

# Process 3: shards 42-63
SHARD_COUNT=22 SHARD_START=42 TOTAL_SHARDS=64 ./dicemaiden-rs &
```

## Differences from Original

This Rust implementation maintains full compatibility with the original DiceMaiden's dice syntax while offering:

- **Better Performance**: Rust's zero-cost abstractions and memory safety plus low memory utilization
- **Modern Discord API**: Native slash command support
- **Type Safety**: Compile-time guarantees prevent runtime errors
- **Easier Deployment**: Single binary with no runtime dependencies
- **Better Error Handling**: Comprehensive error messages and recovery

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## License

This project is licensed under the GPLv3 License

## Acknowledgments

- Original [DiceMaiden](https://github.com/Humblemonk/DiceMaiden) by Humblemonk and many awesome contributors!
- [Serenity](https://github.com/serenity-rs/serenity) Discord library
- The Rust community for excellent crates and documentation

## Support

- Create an issue for bugs or feature requests
- Check the original DiceMaiden documentation for dice syntax questions
- Join the [Discord community](https://discord.gg/AYNcxc9NeU) for help and discussion
