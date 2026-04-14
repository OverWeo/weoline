#!/usr/bin/env bash
set -euo pipefail

REPO="OverWeo/weoline"
BINARY="weoline"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
case "$OS" in
  darwin) OS="apple-darwin" ;;
  linux)  OS="unknown-linux-gnu" ;;
  *)
    echo "Unsupported OS: $OS" >&2
    exit 1
    ;;
esac

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
  x86_64)        ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *)
    echo "Unsupported architecture: $ARCH" >&2
    exit 1
    ;;
esac

TARGET="${ARCH}-${OS}"
ARCHIVE="weoline-${TARGET}.tar.gz"

# Fetch latest release version
echo "Fetching latest version..."
VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep -o '"tag_name": *"[^"]*"' \
  | grep -o '"[^"]*"$' \
  | tr -d '"')

if [ -z "$VERSION" ]; then
  echo "Failed to fetch latest version from GitHub API" >&2
  exit 1
fi

# Validate version format before using it in any interpolation
if ! [[ "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Unexpected version string: '$VERSION'" >&2
  exit 1
fi

echo "Installing weoline ${VERSION} for ${TARGET}..."

# Download to a temp dir; clean up on any exit
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"
echo "Downloading ${URL}..."
curl -fsSL --retry 3 --retry-delay 1 -o "${TMP_DIR}/${ARCHIVE}" "$URL"

# Extract
tar -xzf "${TMP_DIR}/${ARCHIVE}" -C "$TMP_DIR"

# Verify binary is present before installing
if [ ! -f "${TMP_DIR}/${BINARY}" ]; then
  echo "Binary '${BINARY}' not found after extraction. Archive contents:" >&2
  tar -tzf "${TMP_DIR}/${ARCHIVE}" >&2
  exit 1
fi

# Install
mkdir -p "$INSTALL_DIR"
install -m755 "${TMP_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"

echo "Installed weoline ${VERSION} to ${INSTALL_DIR}/${BINARY}"

# PATH hint
case ":${PATH}:" in
  *":${INSTALL_DIR}:"*)
    echo "weoline is ready — run 'weoline --version'"
    ;;
  *)
    echo ""
    echo "NOTE: ${INSTALL_DIR} is not in your PATH."
    echo "Add it to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    ;;
esac
