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

# --- fetch latest version (no API, follows web redirect to avoid rate limits) ---
LATEST=$(curl -Ls -o /dev/null -w '%{url_effective}' "https://github.com/${REPO}/releases/latest")
LATEST=$(basename "${LATEST}")

if [ -z "${LATEST}" ] || [ "${LATEST}" = "releases" ]; then
    echo "failed to determine latest version" >&2
    exit 1
fi
ASSET="mgit-cli-${LATEST}-${TARGET}.${EXT}"

# --- download & install ---
echo "installing mgit CLI ${LATEST} for ${TARGET} ..."

case "${EXT}" in
    tar.gz)
        DEST="${HOME}/.local/bin"
        mkdir -p "${DEST}"
        curl -fSL "https://github.com/${REPO}/releases/download/${LATEST}/${ASSET}" \
            | tar xz --strip-components=1 -C "${DEST}"
        chmod +x "${DEST}/mgit"
        echo "done.  installed to ${DEST}/mgit"
        if ! echo "${PATH}" | grep -q "${DEST}"; then
            echo ""
            echo "add ${DEST} to your PATH:"
            echo "  echo 'export PATH=\"\${HOME}/.local/bin:\${PATH}\"' >> ~/.zshrc"
        fi
        ;;
    zip)
        DEST="${HOME}/.mgit/bin"
        mkdir -p "${DEST}"
        TMP_ZIP="${DEST}/${ASSET}"
        curl -fSL -o "${TMP_ZIP}" "https://github.com/${REPO}/releases/download/${LATEST}/${ASSET}"
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
esac
