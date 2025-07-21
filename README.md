# Discord Bot - Freja

A memory-enabled Discord bot built in Rust that uses OpenAI's GPT-4o to have natural conversations and remember interactions with users.

## Features

- **Conversational AI**: Responds to messages containing "freja" or direct mentions using OpenAI's GPT-4o
- **Persistent Memory**: Remembers past interactions with each user and builds a memory profile over time
- **Context-Aware**: Understands reply chains and provides relevant responses based on conversation context
- **Slash Commands**: Includes `/memory` command to view what the bot remembers about you
- **Asynchronous**: Built with Tokio for efficient async message handling

## Prerequisites

- **Rust**: Version 1.70+ (uses 2024 edition)
- **Discord Bot Token**: Create a bot application in the Discord Developer Portal
- **OpenAI API Key**: Required for AI-powered responses

## Setup

1. **Clone the repository**:

   ```bash
   git clone <repository-url>
   cd discord_bot
   ```

2. **Set environment variables**:

   ```bash
   export DISCORD_TOKEN="your_discord_bot_token"
   export OPENAI_API_KEY="your_openai_api_key"

   # Optional configuration
   export GUILD_ID="123456789"          # For faster command registration during development
   export MEMORY_FILE="memories.json"   # Default: memories.json
   export MAX_RESPONSE_TOKENS="400"     # Default: 400
   export MAX_MEMORY_TOKENS="400"       # Default: 400
   ```

3. **Build and run**:
   ```bash
   cargo build --release
   cargo run
   ```

## Configuration

The bot uses environment variables for configuration:

| Variable              | Required | Default         | Description                              |
| --------------------- | -------- | --------------- | ---------------------------------------- |
| `DISCORD_TOKEN`       | Yes      | -               | Your Discord bot token                   |
| `OPENAI_API_KEY`      | Yes      | -               | Your OpenAI API key                      |
| `GUILD_ID`            | No       | -               | Guild ID for faster command registration |
| `MEMORY_FILE`         | No       | `memories.json` | File to store user memories              |
| `MAX_RESPONSE_TOKENS` | No       | `400`           | Max tokens for AI responses              |
| `MAX_MEMORY_TOKENS`   | No       | `400`           | Max tokens for memory updates            |

## Usage

### Talking to Freja

The bot will respond when:

- Your message contains "freja" (case-insensitive)
- You mention the bot directly with `@BotName`

### Commands

- `/memory` - Shows what Freja remembers about you

### Memory System

Freja automatically builds and updates memories of your interactions:

- Stores personalized memory for each user
- Updates memory after each interaction
- Uses AI to summarize and maintain relevant information
- Memory persists between bot restarts

## Dependencies

- **poise**: Discord bot framework built on serenity
- **serenity**: Discord API library
- **tokio**: Async runtime
- **reqwest**: HTTP client for OpenAI API
- **serde**: Serialization/deserialization
- **chrono**: Date and time handling

## Development

1. **Enable development mode**: Set `GUILD_ID` to register commands faster in your test server
2. **Monitor logs**: The bot outputs status information to stdout/stderr
3. **Memory file**: User memories are stored in JSON format in the configured memory file

