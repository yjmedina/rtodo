#!/usr/bin/env sh
# install.sh — rtodo installer
# Usage: curl -LsSf https://raw.githubusercontent.com/yjmedina/rtodo/main/install.sh | sh
# Override install dir: RTODO_INSTALL_DIR=~/.bin curl -LsSf ... | sh
set -eu

REPO="yjmedina/rtodo"
BIN="rtodo"
INSTALL_DIR="${RTODO_INSTALL_DIR:-$HOME/.local/bin}"

# ── Helpers ──────────────────────────────────────────────────────────────────

say() { printf 'rtodo-install: %s\n' "$1"; }
err() { say "error: $1" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || err "required tool not found: $1"; }

# ── Dependency check ─────────────────────────────────────────────────────────

need curl
need uname
need tar

if command -v sha256sum >/dev/null 2>&1; then
    CHECKSUM_CMD="sha256sum"
elif command -v shasum >/dev/null 2>&1; then
    CHECKSUM_CMD="shasum"
else
    err "No SHA256 tool found (sha256sum or shasum). Cannot verify download."
fi

# ── Platform detection ───────────────────────────────────────────────────────

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)  OS_TRIPLE="unknown-linux-gnu" ;;
    Darwin) OS_TRIPLE="apple-darwin"      ;;
    MINGW*|MSYS*|CYGWIN*)
        err "Windows detected. Download the .zip from: https://github.com/${REPO}/releases/latest"
        ;;
    *) err "Unsupported OS: $OS (supported: Linux, Darwin)" ;;
esac

case "$ARCH" in
    x86_64)        ARCH_TRIPLE="x86_64"  ;;
    arm64|aarch64) ARCH_TRIPLE="aarch64" ;;
    *) err "Unsupported architecture: $ARCH (supported: x86_64, aarch64)" ;;
esac

TARGET="${ARCH_TRIPLE}-${OS_TRIPLE}"

# ── Fetch latest version ──────────────────────────────────────────────────────

say "Fetching latest release..."

VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' \
    | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')"

[ -n "$VERSION" ] || err "Could not determine latest version. Check your internet connection."

say "Latest version: $VERSION"

# ── Detect existing install ───────────────────────────────────────────────────

EXISTING=""
if command -v "$BIN" >/dev/null 2>&1; then
    EXISTING="$(command -v "$BIN")"
    say "Existing install found at: $EXISTING — updating to $VERSION"
else
    say "Installing $BIN $VERSION for $TARGET"
fi

# ── Build URLs ────────────────────────────────────────────────────────────────

ARCHIVE_NAME="${BIN}-${TARGET}.tar.gz"
BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
ARCHIVE_URL="${BASE_URL}/${ARCHIVE_NAME}"
CHECKSUMS_URL="${BASE_URL}/rtodo-sha256sums.txt"

# ── Download to temp dir ──────────────────────────────────────────────────────

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

say "Downloading $ARCHIVE_URL"
curl -fSL --progress-bar "$ARCHIVE_URL" -o "${TMP_DIR}/${ARCHIVE_NAME}" \
    || err "Download failed. Does release $VERSION exist for $TARGET?"

say "Downloading checksums..."
curl -fsSL "$CHECKSUMS_URL" -o "${TMP_DIR}/sha256sums.txt" \
    || err "Checksum file download failed."

# ── Verify SHA256 ─────────────────────────────────────────────────────────────

say "Verifying checksum..."

cd "$TMP_DIR"
grep "${ARCHIVE_NAME}" sha256sums.txt > expected.txt \
    || err "Checksum for ${ARCHIVE_NAME} not found in sha256sums.txt."

if [ "$CHECKSUM_CMD" = "sha256sum" ]; then
    sha256sum --check --quiet expected.txt \
        || err "SHA256 verification FAILED. Download may be corrupt or tampered."
else
    shasum -a 256 --check --quiet expected.txt \
        || err "SHA256 verification FAILED. Download may be corrupt or tampered."
fi

say "Checksum OK."

# ── Extract and install ───────────────────────────────────────────────────────

tar xzf "${TMP_DIR}/${ARCHIVE_NAME}" -C "$TMP_DIR"

EXTRACTED_BIN="${TMP_DIR}/${BIN}"
[ -f "$EXTRACTED_BIN" ] || err "Binary '$BIN' not found in archive."

chmod +x "$EXTRACTED_BIN"
mkdir -p "$INSTALL_DIR"
mv "$EXTRACTED_BIN" "${INSTALL_DIR}/${BIN}"

# ── Done ──────────────────────────────────────────────────────────────────────

say ""
say "$BIN $VERSION installed to ${INSTALL_DIR}/${BIN}"

case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
        say ""
        say "NOTE: ${INSTALL_DIR} is not in your PATH. Add to your shell profile:"
        say "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        ;;
esac

if [ -n "$EXISTING" ]; then
    say "Update complete."
else
    say "Run: $BIN --help"
fi
