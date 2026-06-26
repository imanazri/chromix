#!/bin/sh
# Chromix installer. Detects your platform, downloads the matching binary
# from the latest GitHub release, and installs it to a directory on your PATH.
set -e

REPO="imanazri/chromix"
BIN="chromix"
INSTALL_DIR="${CHROMIX_INSTALL_DIR:-/usr/local/bin}"

err() {
    echo "error: $1" >&2
    exit 1
}

# Detect OS
os="$(uname -s)"
case "$os" in
    Darwin) os="macos" ;;
    Linux) os="linux" ;;
    *) err "unsupported OS: $os. Build from source instead: https://github.com/$REPO" ;;
esac

# Detect architecture
arch="$(uname -m)"
case "$arch" in
    x86_64 | amd64) arch="x86_64" ;;
    arm64 | aarch64) arch="arm64" ;;
    *) err "unsupported architecture: $arch" ;;
esac

# Linux only ships x86_64
if [ "$os" = "linux" ] && [ "$arch" != "x86_64" ]; then
    err "no prebuilt linux binary for $arch. Build from source instead: https://github.com/$REPO"
fi

asset="${BIN}-${os}-${arch}.tar.gz"
url="https://github.com/${REPO}/releases/latest/download/${asset}"

echo "Downloading ${asset}..."
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

if ! curl -fsSL "$url" -o "$tmp/$asset"; then
    err "download failed. Check that a release exists at https://github.com/$REPO/releases"
fi

tar xzf "$tmp/$asset" -C "$tmp"

echo "Installing to ${INSTALL_DIR}..."
if [ -w "$INSTALL_DIR" ]; then
    mv "$tmp/$BIN" "$INSTALL_DIR/$BIN"
else
    sudo mv "$tmp/$BIN" "$INSTALL_DIR/$BIN"
fi

chmod +x "$INSTALL_DIR/$BIN"

echo "Installed $($INSTALL_DIR/$BIN --version)"
echo "Run 'chromix' to get started."
