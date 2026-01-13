#!/bin/bash
set -e

echo "Installing agent-inbox..."

# Build release binary
cargo build --release

# Install binary to /usr/local/bin (requires sudo)
if [ -w "/usr/local/bin" ]; then
    cp target/release/agent-inbox /usr/local/bin/
    echo "✓ Installed agent-inbox to /usr/local/bin/"
else
    sudo cp target/release/agent-inbox /usr/local/bin/
    echo "✓ Installed agent-inbox to /usr/local/bin/ (with sudo)"
fi

# Create data directory
mkdir -p ~/.agent-tasks/wrappers
echo "✓ Created ~/.agent-tasks directory"

echo ""
echo "Installation complete!"
echo ""
echo "Next steps:"
echo "  1. Try: agent-inbox --help"
echo "  2. For Phase 2 (CLI wrappers), run: ./setup-wrappers.sh"
echo ""
