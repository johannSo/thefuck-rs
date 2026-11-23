#!/bin/sh
# Installs The Fuck (the Rust version) by building it from source with cargo.
#
#   curl -fsSL https://raw.githubusercontent.com/johannSo/thefuck-rs/master/install.sh | sh
#
# Set THEFUCK_INSTALL_ROOT to install somewhere else than cargo's default
# location (the binaries go to $THEFUCK_INSTALL_ROOT/bin).

set -eu

REPO="https://github.com/johannSo/thefuck-rs"

if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo is required to build The Fuck, but it isn't installed." >&2
    echo "Install Rust from https://rustup.rs and run this script again." >&2
    exit 1
fi

echo "Building The Fuck from $REPO, this takes about a minute..."
if [ -n "${THEFUCK_INSTALL_ROOT:-}" ]; then
    cargo install --git "$REPO" --locked --force --root "$THEFUCK_INSTALL_ROOT"
    bin_dir="$THEFUCK_INSTALL_ROOT/bin"
else
    cargo install --git "$REPO" --locked --force
    bin_dir="${CARGO_INSTALL_ROOT:-${CARGO_HOME:-$HOME/.cargo}}/bin"
fi

echo
echo "The Fuck is installed to $bin_dir"
case ":$PATH:" in
    *":$bin_dir:"*) ;;
    *) echo "Add it to your PATH:  export PATH=\"$bin_dir:\$PATH\"" ;;
esac

case "$(basename "${SHELL:-sh}")" in
    fish) alias_line='thefuck --alias | source' ; config='~/.config/fish/config.fish' ;;
    zsh) alias_line='eval $(thefuck --alias)' ; config='~/.zshrc' ;;
    tcsh | csh) alias_line='eval `thefuck --alias`' ; config='~/.tcshrc' ;;
    *) alias_line='eval "$(thefuck --alias)"' ; config='~/.bashrc' ;;
esac
echo "To finish, put this line in your $config and restart your shell:"
echo
echo "    $alias_line"
echo
echo "Or just run \`fuck\` twice, it configures the alias itself."
