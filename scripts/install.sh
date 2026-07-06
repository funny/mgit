#!/usr/bin/env bash
set -euo pipefail

REPO="yhx0516/mgit"
UNAME_S=$(uname -s | tr '[:upper:]' '[:lower:]')
UNAME_M=$(uname -m)

# Pick the platform-specific installer asset pattern
case "${UNAME_S}-${UNAME_M}" in
    darwin-*)
        PATTERN="unified.dmg"
        OPEN_CMD="open"
        ;;
    linux-x86_64|linux-aarch64)
        PATTERN="_amd64.deb"
        OPEN_CMD="xdg-open"
        ;;
    mingw*-x86_64|msys*-x86_64|cygwin*-x86_64)
        PATTERN="setup.exe"
        OPEN_CMD="start"
        ;;
    *)
        echo "unsupported platform: ${UNAME_S}-${UNAME_M}" >&2
        exit 1
        ;;
esac

# Fetch the latest release and find a matching asset
echo "checking latest release ..."
RELEASE_JSON=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest")

# Extract asset info with a lightweight grep+sed approach (no jq dependency)
ASSET_NAME=$(echo "${RELEASE_JSON}" \
    | grep -o '"name": *"[^"]*"' \
    | sed -E 's/.*"([^"]+)".*/\1/' \
    | grep "${PATTERN}" \
    | head -1)

if [ -z "${ASSET_NAME}" ]; then
    echo "no GUI installer found for ${PATTERN}" >&2
    exit 1
fi

DOWNLOAD_URL=$(echo "${RELEASE_JSON}" \
    | grep -o '"browser_download_url": *"[^"]*"' \
    | grep "${ASSET_NAME}" \
    | sed -E 's/.*"(https:[^"]+)".*/\1/' \
    | head -1)

DEST_DIR="${HOME}/.mgit/updates"
mkdir -p "${DEST_DIR}"
DEST_FILE="${DEST_DIR}/${ASSET_NAME}"

echo "downloading ${ASSET_NAME} ..."
curl -fSL --progress-bar -o "${DEST_FILE}" "${DOWNLOAD_URL}"

echo "opening installer ..."
case "${OPEN_CMD}" in
    open)   open "${DEST_FILE}" ;;
    xdg-open) xdg-open "${DEST_FILE}" ;;
    start)  cmd /c start "" "${DEST_FILE}" ;;
esac

echo "done. follow the installer prompts to complete installation."
