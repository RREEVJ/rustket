#!/usr/bin/env bash
set -euo pipefail

VERSION="${RUSTKET_VERSION:-0.1.0}"
REPOSITORY="Rustket/rustket"
RELEASE_URL="https://github.com/${REPOSITORY}/releases/download/${VERSION}"
INSTALL_DIR="${HOME:?HOME must be set}/.rustket/bin"

die() {
    printf 'rustket installer: %s\n' "$*" >&2
    exit 1
}

for command in curl tar awk git; do
    command -v "$command" >/dev/null 2>&1 || die "required command not found: $command"
done

if ! command -v shasum >/dev/null 2>&1 && ! command -v sha256sum >/dev/null 2>&1; then
    die "install shasum or sha256sum to verify the download"
fi

os="$(uname -s)"
arch="$(uname -m)"
case "${os}:${arch}" in
    Darwin:arm64|Darwin:aarch64) platform="macos-aarch64" ;;
    Darwin:x86_64) platform="macos-x86_64" ;;
    Linux:arm64|Linux:aarch64) platform="linux-aarch64" ;;
    Linux:x86_64|Linux:amd64) platform="linux-x86_64" ;;
    *) die "no release build for ${os}-${arch}" ;;
esac

archive="rustket-${VERSION}-${platform}.tar.gz"
tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/rustket-install.XXXXXX")"
tmp_binary=""
cleanup() {
    rm -rf "$tmp_dir"
    if [ -n "$tmp_binary" ]; then
        rm -f "$tmp_binary"
    fi
}
trap cleanup EXIT

curl -fsSL "${RELEASE_URL}/SHA256SUMS" -o "${tmp_dir}/SHA256SUMS"
curl -fsSL "${RELEASE_URL}/${archive}" -o "${tmp_dir}/${archive}"

expected="$(awk -v name="$archive" '$2 == name { print $1; found = 1 } END { if (!found) exit 1 }' "${tmp_dir}/SHA256SUMS")" \
    || die "no checksum found for ${archive}"
[[ "$expected" =~ ^[[:xdigit:]]{64}$ ]] || die "invalid checksum for ${archive}"

if command -v shasum >/dev/null 2>&1; then
    actual="$(shasum -a 256 "${tmp_dir}/${archive}" | awk '{ print $1 }')"
else
    actual="$(sha256sum "${tmp_dir}/${archive}" | awk '{ print $1 }')"
fi
[ "$actual" = "$expected" ] || die "SHA-256 verification failed for ${archive}"

tar -xzf "${tmp_dir}/${archive}" -C "$tmp_dir" rustket
[ -f "${tmp_dir}/rustket" ] || die "archive does not contain the rustket executable"

tap_dir="${HOME}/.rustket/taps/Rustket/rustket-bits"
if [ -e "$tap_dir" ]; then
    [ -d "$tap_dir" ] || die "tap path exists and is not a directory: ${tap_dir}"
    printf 'Official tap is already present.\n'
else
    mkdir -p "$(dirname "$tap_dir")"
    git clone --depth 1 -- https://github.com/Rustket/rustket-bits.git "$tap_dir"
fi

mkdir -p "$INSTALL_DIR"
target="${INSTALL_DIR}/rustket"
if [ -d "$target" ] && [ ! -L "$target" ]; then
    die "cannot replace directory at ${target}"
fi
tmp_binary="${INSTALL_DIR}/.rustket.$$"
install -m 755 "${tmp_dir}/rustket" "$tmp_binary"
mv -f "$tmp_binary" "$target"
tmp_binary=""

path_line='export PATH="$HOME/.rustket/bin:$PATH"'
shell_name="${SHELL:-}"
shell_name="${shell_name##*/}"
case "$shell_name" in
    bash)
        rc_file="${HOME}/.bashrc"
        ;;
    zsh)
        rc_file="${HOME}/.zshrc"
        ;;
    fish)
        rc_file="${HOME}/.config/fish/config.fish"
        path_line='fish_add_path "$HOME/.rustket/bin"'
        ;;
    *)
        rc_file=""
        ;;
esac

if [ -n "$rc_file" ] && ! grep -Fqx "$path_line" "$rc_file" 2>/dev/null; then
    mkdir -p "$(dirname "$rc_file")"
    printf '\n%s\n' "$path_line" >> "$rc_file"
    printf 'Added Rustket to PATH in %s.\n' "$rc_file"
fi

printf 'Installed Rustket %s for %s.\n' "$VERSION" "$platform"
case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
        case "$shell_name" in
            bash|zsh)
                printf 'To use it in this shell now, run:\n  export PATH="%s:$PATH"\n' "$INSTALL_DIR"
                ;;
            fish)
                printf 'To use it in this shell now, run:\n  fish_add_path "%s"\n' "$INSTALL_DIR"
                ;;
            *)
                printf 'Add %s to your shell PATH, then open a new shell.\n' "$INSTALL_DIR"
                ;;
        esac
        ;;
esac
