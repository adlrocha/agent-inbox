#!/bin/bash
set -e

echo "Setting up Agent Inbox Extension for Brave Browser..."
echo ""

# Build agent-bridge if not already built
if [ ! -f "target/release/agent-bridge" ]; then
    echo "Building agent-bridge binary..."
    cargo build --release --bin agent-bridge
fi

# Install agent-bridge
AGENT_BRIDGE_PATH="/usr/local/bin/agent-bridge"

if [ -w "/usr/local/bin" ]; then
    cp target/release/agent-bridge "$AGENT_BRIDGE_PATH"
    echo "✓ Installed agent-bridge to $AGENT_BRIDGE_PATH"
else
    echo "Installing agent-bridge requires sudo..."
    sudo cp target/release/agent-bridge "$AGENT_BRIDGE_PATH"
    echo "✓ Installed agent-bridge to $AGENT_BRIDGE_PATH (with sudo)"
fi

# Create Brave NativeMessagingHosts directory
BRAVE_MANIFEST_DIR="$HOME/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts"
mkdir -p "$BRAVE_MANIFEST_DIR"
echo "✓ Created directory: $BRAVE_MANIFEST_DIR"

# Create manifest file with actual path
cat > "$BRAVE_MANIFEST_DIR/com.agent_tasks.bridge.json" << EOF
{
  "name": "com.agent_tasks.bridge",
  "description": "Native messaging host for Agent Inbox extension",
  "path": "$AGENT_BRIDGE_PATH",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://EXTENSION_ID_PLACEHOLDER/"
  ]
}
EOF

echo "✓ Created manifest: $BRAVE_MANIFEST_DIR/com.agent_tasks.bridge.json"
echo ""
echo "============================================"
echo "Setup Complete!"
echo "============================================"
echo ""
echo "Next steps:"
echo ""
echo "1. Load the extension in Brave:"
echo "   - Open: brave://extensions"
echo "   - Enable 'Developer mode' (toggle in top-right)"
echo "   - Click 'Load unpacked'"
echo "   - Select: $(pwd)/extension"
echo ""
echo "2. Copy the Extension ID:"
echo "   - After loading, you'll see an ID like:"
echo "     abcdefghijklmnopqrstuvwxyz123456"
echo "   - Copy it to clipboard"
echo ""
echo "3. Update the manifest with your Extension ID:"
echo "   - Run this command (replace YOUR_ID with actual ID):"
echo ""
echo "     sed -i 's/EXTENSION_ID_PLACEHOLDER/YOUR_ACTUAL_ID_HERE/' \\"
echo "       $BRAVE_MANIFEST_DIR/com.agent_tasks.bridge.json"
echo ""
echo "   - Or edit manually:"
echo "     nano $BRAVE_MANIFEST_DIR/com.agent_tasks.bridge.json"
echo ""
echo "4. Reload the extension:"
echo "   - Go back to brave://extensions"
echo "   - Click the reload button on 'Agent Inbox Tracker'"
echo ""
echo "5. Check if it's working:"
echo "   - Click 'background page' on the extension"
echo "   - Console should show: 'Connected to native host: com.agent_tasks.bridge'"
echo ""
echo "6. Test it:"
echo "   - Open https://claude.ai and start a conversation"
echo "   - Run: agent-inbox list --all"
echo "   - Your conversation should appear!"
echo ""
echo "Troubleshooting:"
echo "   - If you see 'Specified native messaging host not found':"
echo "     Make sure you completed step 3 (updating Extension ID)"
echo ""
echo "   - Manifest file location:"
echo "     $BRAVE_MANIFEST_DIR/com.agent_tasks.bridge.json"
echo ""
