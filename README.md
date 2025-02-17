# GlebusVPN Bot 🤖

Telegram bot for managing VPN connections to GlebusVPN.

## Features ✨
- Device management (1-5 devices)
- Interactive dialogue system
- Cancellable operations

## Installation 📦

1. Clone repository:
```bash
git clone https://github.com/yourusername/glebus-vpn-bot.git
cd glebus-vpn-bot
```

2. Create .env file:
```
TELOXIDE_TOKEN=your_bot_token
LOG_CONFIG=path_to_log4rs.yaml
PANEL_BASE_URL=http://0.0.0.0:00000/panel
SUB_BASE_URL=http://0.0.0.0:00000/sub
PANEL_ADMIN_LOGIN=your_login_to_panel
PANEL_ADMIN_PASSWORD=your_password_to_panel
```

3. Build and run:
```bash
cargo run --release
```
