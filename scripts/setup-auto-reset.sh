#!/bin/bash
# Setup automatic reset of agent-inbox tasks on login/restart
# This creates a systemd user service that clears stale tasks

set -e

AGENT_INBOX_BIN="${AGENT_INBOX_BIN:-$HOME/.local/bin/agent-inbox}"
SERVICE_DIR="$HOME/.config/systemd/user"
SERVICE_FILE="$SERVICE_DIR/agent-inbox-reset.service"

# Check if agent-inbox is installed
if [ ! -x "$AGENT_INBOX_BIN" ]; then
    echo "Error: agent-inbox not found at $AGENT_INBOX_BIN"
    echo "Please install it first or set AGENT_INBOX_BIN environment variable"
    exit 1
fi

# Create systemd user directory
mkdir -p "$SERVICE_DIR"

# Create the service file
cat > "$SERVICE_FILE" << EOF
[Unit]
Description=Reset agent-inbox tasks on login
After=default.target

[Service]
Type=oneshot
ExecStart=$AGENT_INBOX_BIN reset --force
RemainAfterExit=yes

[Install]
WantedBy=default.target
EOF

echo "Created service file: $SERVICE_FILE"

# Reload systemd and enable the service
systemctl --user daemon-reload
systemctl --user enable agent-inbox-reset.service

echo ""
echo "Auto-reset service installed and enabled!"
echo "Tasks will be automatically cleared on each login/restart."
echo ""
echo "To disable: systemctl --user disable agent-inbox-reset.service"
echo "To check status: systemctl --user status agent-inbox-reset.service"
