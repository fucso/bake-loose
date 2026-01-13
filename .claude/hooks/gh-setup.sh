#!/bin/bash
# gh CLI setup script for Claude Code on the Web
# This script is executed as a sessionStartHook to install gh CLI in remote environments

set -e

# Only run in remote environments (Claude Code on the Web)
if [ "$CLAUDE_CODE_REMOTE" != "true" ]; then
    exit 0
fi

# Check if gh is already installed
if command -v gh &> /dev/null; then
    exit 0
fi

# Create local bin directory
LOCAL_BIN="$HOME/.local/bin"
mkdir -p "$LOCAL_BIN"

# Download and install gh CLI
GH_VERSION="2.63.2"
ARCH="amd64"

echo "Installing gh CLI v${GH_VERSION}..."

# Download gh CLI
curl -sL "https://github.com/cli/cli/releases/download/v${GH_VERSION}/gh_${GH_VERSION}_linux_${ARCH}.tar.gz" -o /tmp/gh.tar.gz

# Extract and install
tar -xzf /tmp/gh.tar.gz -C /tmp
cp "/tmp/gh_${GH_VERSION}_linux_${ARCH}/bin/gh" "$LOCAL_BIN/gh"
chmod +x "$LOCAL_BIN/gh"

# Cleanup
rm -rf /tmp/gh.tar.gz "/tmp/gh_${GH_VERSION}_linux_${ARCH}"

# Add to PATH for this session via CLAUDE_ENV_FILE
if [ -n "$CLAUDE_ENV_FILE" ]; then
    echo "PATH=$LOCAL_BIN:\$PATH" >> "$CLAUDE_ENV_FILE"
fi

# Also export for current shell
export PATH="$LOCAL_BIN:$PATH"

echo "gh CLI installed successfully!"
gh --version
