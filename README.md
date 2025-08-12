# GlebusVPN Bot 🤖

Telegram bot for managing VPN connections to GlebusVPN.

A Rust-based Telegram bot for managing VPN subscriptions via the [Remnawave API](https://github.com/remnawave/rust-sdk). The bot allows users to create, view, recreate, and delete their VPN subscriptions directly from Telegram.

## Features

- 🚀 Create new VPN subscriptions
- 🔑 View existing subscription links
- 🔄 Regenerate subscription links
- ❌ Delete subscriptions
- ℹ️ View detailed user/profile information
- 📊 Monitor traffic usage
- 📝 Comprehensive error handling

## Requirements

1. Rust 1.70+
2. Telegram Bot API token
3. Remnawave API access credentials
4. Environment variables configured (see setup)

## Installation & Setup 📦

### 1. Clone repository
```bash
git clone https://github.com/LineGM/glebus_vpn_bot.git
cd glebus_vpn_bot
```

### 2. Build project
```bash
cargo build --release
```

### 2. Create .env file
Create .env file in the project root directory (same level as Cargo.toml) with:
```
TELOXIDE_TOKEN=your_telegram_bot_token
PANEL_BASE_URL=https://your.panel.url
REMNAWAVE_API_TOKEN=your_remnawave_api_token
```
When running the compiled binary directly, place .env in the same directory as the executable:

/target/release/

├── glebus_vpn_bot  # Binary

└── .env            # Environment file

3. Run:
```bash
# Cargo
cargo run --release

# Binary directly
cd target/release
./glebus_vpn_bot
```
