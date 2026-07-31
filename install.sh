#!/bin/sh
# Installs the latest spnd release binary for this machine.
#
#   curl -fsSL https://raw.githubusercontent.com/Royalflamejlh/spnd/main/install.sh | sh
#
# Environment:
#   SPND_INSTALL_DIR  target directory (default: ~/.local/bin)
#   SPND_VERSION      tag to install, e.g. v21.0.0 (default: latest release)
set -eu

repo="Royalflamejlh/spnd"

case "$(uname -s)" in
	Linux) platform="linux" ;;
	Darwin) platform="darwin" ;;
	*)
		echo "error: unsupported OS $(uname -s); on Windows use Scoop (scoop bucket add spnd https://github.com/$repo) or npm (npm i -g @spnd/spnd)" >&2
		exit 1
		;;
esac
case "$(uname -m)" in
	x86_64 | amd64) arch="x64" ;;
	aarch64 | arm64) arch="arm64" ;;
	*)
		echo "error: unsupported architecture $(uname -m)" >&2
		exit 1
		;;
esac

asset="spnd-$platform-$arch.tar.gz"
if [ -n "${SPND_VERSION:-}" ]; then
	url="https://github.com/$repo/releases/download/$SPND_VERSION/$asset"
else
	url="https://github.com/$repo/releases/latest/download/$asset"
fi

install_dir="${SPND_INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "$install_dir"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

echo "downloading $url"
curl -fsSL -o "$tmp/$asset" "$url"
tar -xzf "$tmp/$asset" -C "$tmp" spnd
install -m 755 "$tmp/spnd" "$install_dir/spnd"

echo "installed $("$install_dir/spnd" --version) to $install_dir/spnd"
case ":$PATH:" in
	*":$install_dir:"*) ;;
	*) echo "note: $install_dir is not on your PATH" ;;
esac
