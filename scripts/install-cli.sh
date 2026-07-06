#!/usr/bin/env bash
set -euo pipefail

REPO="yhx0516/mgit"
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# --- platform detection ---
case "${OS}" in
    linux)
        case "${ARCH}" in
            x86_64)  TARGET="x86_64-unknown-linux-musl"; EXT="tar.gz" ;;
            aarch64) TARGET="aarch64-unknown-linux-musl"; EXT="tar.gz" ;;
            *) echo "unsupported arch: ${ARCH}" >&2; exit 1 ;;
        esac
        ;;
    darwin)
        case "${ARCH}" in
            x86_64) TARGET="x86_64-apple-darwin"; EXT="tar.gz" ;;
            arm64|aarch64) TARGET="aarch64-apple-darwin"; EXT="tar.gz" ;;
            *) echo "unsupported arch: ${ARCH}" >&2; exit 1 ;;
        esac
        ;;
    mingw*|msys*|cygwin*)
        # Windows via Git Bash / MSYS2
        case "${ARCH}" in
            x86_64) TARGET="x86_64-pc-windows-msvc"; EXT="zip" ;;
            *) echo "unsupported arch: ${ARCH}" >&2; exit 1 ;;
        esac
        ;;
    *)
        echo "unsupported platform: ${OS}-${ARCH}" >&2; exit 1 ;;
esac

# --- fetch latest version ---
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
ASSET="mgit-cli-${LATEST}-${TARGET}.${EXT}"

# --- download & install ---
echo "installing mgit CLI ${LATEST} for ${TARGET} ..."

case "${EXT}" in
    tar.gz)
        DEST="/usr/local/bin"
        curl -fSL --progress-bar "https://github.com/${REPO}/releases/download/${LATEST}/${ASSET}" \
            | sudo tar xz -C "${DEST}"
        sudo chmod +x "${DEST}/mgit"
        echo "done.  run 'mgit --version' to verify."
        ;;
    zip)
        DEST="${HOME}/.mgit/bin"
        mkdir -p "${DEST}"
        TMP_ZIP="${DEST}/${ASSET}"
        curl -fSL --progress-bar -o "${TMP_ZIP}" "https://github.com/${REPO}/releases/download/${LATEST}/${ASSET}"
        unzip -o -d "${DEST}" "${TMP_ZIP}"
        rm -f "${TMP_ZIP}"
        echo "done.  installed to ${DEST}/mgit.exe"

        # Try to add to user PATH automatically (higher than elevated prompt).
        WIN_DEST=$(echo "${DEST}" | sed 's|/|\\|g')
        if powershell.exe -NoProfile -Command \
            "[Environment]::SetEnvironmentVariable('PATH', [Environment]::GetEnvironmentVariable('PATH','User') + ';${WIN_DEST}', 'User')" 2>/dev/null; then
            echo "added to your user PATH — restart your terminal for it to take effect."
        else
            echo ""
            echo "to add mgit to your PATH, run in PowerShell:"
            echo "  [Environment]::SetEnvironmentVariable('PATH', \$env:PATH + ';${WIN_DEST}', 'User')"
        fi
        ;;
