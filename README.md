<div align="center">
  <h1>Freja: AI-Powered Discord Bot</h1>
</div>

<table style="border: none; border-collapse: collapse;">
<tr style="border: none;">
<td width="60%" style="border: none;">

## About Freja

A Discord bot with persistent memory that remembers conversations with users, providing personalized and contextual interactions. Freja creates a more natural and engaging Discord experience by maintaining context across sessions and providing thoughtful, personalized responses.

## ✨ Features

- **🧠 Persistent Memory**: Maintains AI-generated summaries of conversations with each user across sessions
- **💬 Natural Interactions**: Responds when mentioned or when its name appears in messages  
- **🔄 Contextual Responses**: Understands message replies and maintains conversation flow
- **🔍 Memory Inspection**: Users can view what the bot remembers about them via slash command
- **⚙️ Configurable**: Flexible configuration through environment variables

</td>
<td width="40%" style="border: none;">
<div align="center">
  <img src="assets/freja_logo.png" width="100%" style="border-radius: 15px; max-width: 300px;" />
</div>
</td>
</tr>
</table>

---

## Installation

### Prerequisites

- Rust 1.70 or higher
- A Discord bot token from the [Discord Developer Portal](https://discord.com/developers/applications)
- An OpenAI API key from [OpenAI Platform](https://platform.openai.com/)

### Setup

1. Clone the repository:
```bash
git clone https://gitlab.com/eddiebrunstedt/freja.git
cd freja
```

2. Create a `.env` file or set environment variables:
```bash
# Required
DISCORD_TOKEN=your_discord_bot_token
OPENAI_API_KEY=your_openai_api_key

# Optional
GUILD_ID=your_guild_id              # For faster command registration during development
MAX_MEMORY_TOKENS=400               # Token limit for user memory summaries
MAX_RESPONSE_TOKENS=400             # Token limit for bot responses
MEMORY_FILE=memories.json           # Path to memory persistence file
PRETTY_LOG=false                    # Enable pretty logging for development
RUST_LOG=info                       # Logging level
```

3. Build and run:
```bash
cargo build --release
cargo run --release
```

## Usage

### Interacting with Freja

The bot responds in two ways:

1. **Direct Mention**: `@Freja how are you today?`
2. **Name in Message**: `Hey freja, what do you think about this?`

### Commands

- `/memory` - Shows what Freja remembers about you from past conversations

### Memory System

Freja maintains conversation context by:
- Storing AI-generated summaries of interactions with each user
- Updating memories after each conversation
- Persisting memories to disk for reliability across restarts
- Using memories to provide personalized responses

## Configuration

All configuration is handled through environment variables:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DISCORD_TOKEN` | Yes | - | Discord bot authentication token |
| `OPENAI_API_KEY` | Yes | - | OpenAI API key for GPT-4 access |
| `GUILD_ID` | No | - | Specific guild ID for command registration |
| `MAX_MEMORY_TOKENS` | No | 400 | Maximum tokens for memory summaries |
| `MAX_RESPONSE_TOKENS` | No | 400 | Maximum tokens for bot responses |
| `MEMORY_FILE` | No | memories.json | Path to memory storage file |
| `PRETTY_LOG` | No | false | Enable human-readable logging |
| `RUST_LOG` | No | info | Logging level (error, warn, info, debug, trace) |

## Architecture

The bot is built with:
- **[Poise](https://github.com/serenity-rs/poise)**: Discord bot framework
- **[Serenity](https://github.com/serenity-rs/serenity)**: Discord API library
- **[Tokio](https://tokio.rs/)**: Async runtime
- **[OpenAI API](https://platform.openai.com/)**: GPT-4 for responses and memory management

### Project Structure

```
src/
├── ai/                 # OpenAI integration
│   ├── client.rs      # API client implementation
│   └── prompts.rs     # Prompt templates
├── commands/          # Slash commands
│   └── memory.rs      # Memory inspection command
├── handlers/          # Event handlers
│   └── message.rs     # Message processing
├── memory/            # Memory persistence
│   └── store.rs       # Storage implementation
├── config.rs          # Configuration management
├── data.rs           # Shared application state
├── main.rs           # Application entry point
└── tracing.rs        # Logging setup
```

## Development

### Running Tests

```bash
cargo test
```

### Building for Production

```bash
cargo build --release
```

### Logging

Control logging verbosity with the `RUST_LOG` environment variable:
- `RUST_LOG=debug` for detailed debugging
- `RUST_LOG=freja=debug` for debug logs only from this application

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

Eddie Brunstedt <eddie.brunstedt@protonmail.com>

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
