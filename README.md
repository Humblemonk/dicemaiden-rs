# Dice Maiden - Rust Edition

A powerful Discord dice rolling bot written in Rust using the Serenity framework. This is a complete rewrite of the original [DiceMaiden](https://github.com/Humblemonk/DiceMaiden) Ruby bot, featuring all the same functionality with improved performance and memory safety.

## Features

- **Comprehensive Dice Rolling**: Support for complex dice expressions with modifiers
- **Game System Aliases**: Built-in support for popular RPG systems
- **Slash Commands**: Modern Discord integration with `/roll` and `/r` commands
- **Advanced Modifiers**: Exploding dice, keep/drop, rerolls, success counting, and more
- **Multiple Roll Types**: Single rolls, roll sets, and multi-roll expressions
- **Message Management**: Purge command for cleaning up chat

## Quick Start

1. **Clone the repository**
   ```bash
   git clone <your-repo-url>
   cd dicemaiden-rs
   ```

2. **Set up environment**
   ```bash
   cp env.example .env
   # Edit .env and add your Discord bot token
   ```

3. **Build and run**
   ```bash
   cargo build --release
   cargo run
   ```

## Discord Bot Setup

1. Go to the [Discord Developer Portal](https://discord.com/developers/applications)
2. Create a new application and bot
3. Copy the bot token to your `.env` file
4. Invite the bot to your server with the following permissions:
   - Send Messages
   - Use Slash Commands
   - Manage Messages (for purge command)
   - Read Message History

**Invite URL Template:**
```
https://discord.com/api/oauth2/authorize?client_id=YOUR_BOT_ID&permissions=274878000128&scope=bot%20applications.commands
```

## Dice Rolling Syntax

### Basic Usage
- `/roll 2d6` - Roll two six-sided dice
- `/roll 3d6 + 5` - Roll 3d6 and add 5
- `/roll 4d6 k3` - Roll 4d6, keep highest 3

### Modifiers
- **Exploding**: `e6` (explode on 6), `ie6` (explode indefinitely)
- **Keep/Drop**: `k3` (keep 3 highest), `kl2` (keep 2 lowest), `d1` (drop 1 lowest)
- **Rerolls**: `r2` (reroll ≤2 once), `ir2` (reroll ≤2 indefinitely)
- **Success Counting**: `t7` (count successes ≥7), `f1` (count failures ≤1)
- **Math**: `+5`, `-3`, `*2`, `/2`

### Special Features
- **Roll Sets**: `/roll 6 4d6` (roll 6 sets of 4d6)
- **Multi-Roll**: `/roll 2d6 ; 3d8 ; 1d20` (separate rolls)
- **Comments**: `/roll 2d6 ! Fire damage`
- **Labels**: `/roll (Attack) 1d20 + 5`

### Game System Aliases

#### D&D/Pathfinder
- `dndstats` → 6 4d6 k3
- `attack +5` → 1d20 +5
- `+d20` → 2d20 k1 (advantage)
- `-d20` → 2d20 kl1 (disadvantage)

#### World of Darkness
- `4cod` → 4d10 t8 ie10 (Chronicles of Darkness)
- `4wod8` → 4d10 f1 ie10 t8 (World of Darkness)

#### Other Systems
- `sr6` → 6d6 t5 (Shadowrun)
- `ex5` → 5d10 t7 t10 (Exalted)
- `3df` → 3d3 t3 f1 (Fudge dice)
- `age` → 2d6 + 1d6 (AGE system)
- `ed15` → Earthdawn step 15

## Commands

- `/roll <dice>` - Roll dice using RPG notation
- `/r <dice>` - Short alias for roll
- `/help [topic]` - Show help (topics: basic, alias, system)
- `/purge <count>` - Delete recent messages (requires permissions)

## Configuration

### Environment Variables
- `DISCORD_TOKEN` - Your Discord bot token (required)
- `GUILD_ID` - Guild ID for testing commands (optional)
- `RUST_LOG` - Log level (default: info)

### Features
The bot includes all features by default. You can customize the build by modifying `Cargo.toml` dependencies.

## Development

### Requirements
- Rust 1.70+ 
- Discord bot token
- A sqlite db residing in the same directory as your .env file

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
```
src/
├── main.rs              # Application entry point
├── dice/
│   ├── mod.rs          # Dice module exports
│   ├── parser.rs       # Dice expression parsing
│   ├── roller.rs       # Dice rolling logic
│   └── aliases.rs      # Game system aliases
└── commands/
    ├── mod.rs          # Command module exports
    ├── roll.rs         # Roll command implementation
    ├── help.rs         # Help command
    └── purge.rs        # Message purge command
```

## Deployment

### Docker
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/dicemaiden-rs /usr/local/bin/
CMD ["dicemaiden-rs"]
```

### Systemd Service
```ini
[Unit]
Description=Dice Maiden Rust Bot
After=network.target

[Service]
Type=simple
User=dicebot
WorkingDirectory=/opt/dicemaiden-rs
Environment=RUST_LOG=info
EnvironmentFile=/opt/dicemaiden-rs/.env
ExecStart=/opt/dicemaiden-rs/target/release/dicemaiden-rs
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
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
- Join the Discord community for help and discussion
