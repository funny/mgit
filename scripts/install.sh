#!/usr/bin/env bash
set -euo pipefail

REPO="yhx0516/mgit"
UNAME_S=$(uname -s | tr '[:upper:]' '[:lower:]')
UNAME_M=$(uname -m)

# --- platform detection + asset name template ---
case "${UNAME_S}-${UNAME_M}" in
    darwin-*)
        ASSET_NAME="mgit-__TAG__-universal.dmg"
        OPEN_CMD="open"
        ;;
    linux-x86_64|linux-aarch64)
        ASSET_NAME="mgit___TAG___amd64.deb"
        OPEN_CMD="xdg-open"
        ;;
    mingw*-x86_64|msys*-x86_64|cygwin*-x86_64)
        ASSET_NAME="mgit-__TAG__-setup.exe"
        OPEN_CMD="cmd //c start \"\""
        ;;
    *)
        echo "unsupported platform: ${UNAME_S}-${UNAME_M}" >&2
        exit 1
        ;;
esac

# --- fetch latest version (no API, follows web redirect to avoid rate limits) ---
echo "checking latest release ..."
LATEST=$(curl -Ls -o /dev/null -w '%{url_effective}' "https://github.com/${REPO}/releases/latest")
LATEST=$(basename "${LATEST}")

if [ -z "${LATEST}" ] || [ "${LATEST}" = "releases" ]; then
    echo "failed to determine latest version" >&2
    exit 1
fi

ASSET_NAME="${ASSET_NAME//__TAG__/${LATEST}}"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST}/${ASSET_NAME}"

# --- download ---
DEST_DIR="${HOME}/.mgit/updates"
mkdir -p "${DEST_DIR}"
DEST_FILE="${DEST_DIR}/${ASSET_NAME}"

echo "downloading ${ASSET_NAME} ..."
curl -fSL -o "${DEST_FILE}" "${DOWNLOAD_URL}"

# --- open installer ---
echo "opening installer ..."
${OPEN_CMD} "${DEST_FILE}"

echo "done. follow the installer prompts to complete installation."
