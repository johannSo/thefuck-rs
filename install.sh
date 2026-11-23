#!/bin/sh
# Installs The Fuck (the Rust version) by building it from source with cargo,
# and sets up the `fuck` alias in the config of your shell.
#
#   curl -fsSL https://raw.githubusercontent.com/johannSo/thefuck-rs/master/install.sh | sh
#
# Set THEFUCK_INSTALL_ROOT to install somewhere else than cargo's default
# location (the binaries go to $THEFUCK_INSTALL_ROOT/bin).
# Set THEFUCK_NO_SHELL_CONFIG=1 to leave the config of your shell alone, and
# THEFUCK_NO_NEW_SHELL=1 to not start a fresh shell when it's done.

set -eu

REPO="https://github.com/johannSo/thefuck-rs"

os="$(uname -s)"

# Right after installing Rust, cargo isn't in PATH of the running shell yet.
cargo="$(command -v cargo 2>/dev/null || true)"
if [ -z "$cargo" ] && [ -x "${CARGO_HOME:-$HOME/.cargo}/bin/cargo" ]; then
    cargo="${CARGO_HOME:-$HOME/.cargo}/bin/cargo"
fi
if [ -z "$cargo" ]; then
    echo "cargo is required to build The Fuck, but it isn't installed." >&2
    echo "Install Rust from https://rustup.rs and run this script again." >&2
    exit 1
fi

# On macOS cargo needs the linker of the Xcode Command Line Tools. Without
# them the build fails, or macOS pops up a dialog in the middle of it.
if [ "$os" = Darwin ] && ! xcode-select -p >/dev/null 2>&1; then
    echo "The Xcode Command Line Tools are required to build The Fuck." >&2
    echo "Install them with  xcode-select --install  and run this script again." >&2
    exit 1
fi

echo "Building The Fuck from $REPO, this takes about a minute..."
if [ -n "${THEFUCK_INSTALL_ROOT:-}" ]; then
    "$cargo" install --git "$REPO" --locked --force --root "$THEFUCK_INSTALL_ROOT"
    bin_dir="$THEFUCK_INSTALL_ROOT/bin"
else
    "$cargo" install --git "$REPO" --locked --force
    bin_dir="${CARGO_INSTALL_ROOT:-${CARGO_HOME:-$HOME/.cargo}}/bin"
fi

echo
echo "The Fuck is installed to $bin_dir"

case ":$PATH:" in
    *":$bin_dir:"*) path_line='' ;;
    *) path_line=1 ;;
esac

shell_name="$(basename "${SHELL:-sh}")"
case "$shell_name" in
    fish)
        config="$HOME/.config/fish/config.fish"
        alias_line='thefuck --alias | source'
        [ -n "$path_line" ] && path_line="set -gx PATH \"$bin_dir\" \$PATH"
        ;;
    zsh)
        config="${ZDOTDIR:-$HOME}/.zshrc"
        alias_line='eval $(thefuck --alias)'
        [ -n "$path_line" ] && path_line="export PATH=\"$bin_dir:\$PATH\""
        ;;
    tcsh | csh)
        config="$HOME/.tcshrc"
        alias_line='eval `thefuck --alias`'
        [ -n "$path_line" ] && path_line="setenv PATH \"$bin_dir:\$PATH\""
        ;;
    *)
        # The same file `fuck` itself would pick.
        if [ "$os" = Darwin ]; then
            # Terminals of macOS start login shells, they read only the first
            # existing of these, never ~/.bashrc. A new ~/.bash_profile would
            # hide an existing ~/.profile from bash, so that one is used then.
            config="$HOME/.bash_profile"
            for login_config in .bash_profile .bash_login .profile; do
                if [ -f "$HOME/$login_config" ]; then
                    config="$HOME/$login_config"
                    break
                fi
            done
        elif [ -f "$HOME/.bashrc" ] || [ ! -f "$HOME/.bash_profile" ]; then
            config="$HOME/.bashrc"
        else
            config="$HOME/.bash_profile"
        fi
        alias_line='eval "$(thefuck --alias)"'
        [ -n "$path_line" ] && path_line="export PATH=\"$bin_dir:\$PATH\""
        ;;
esac

if [ -n "${THEFUCK_NO_SHELL_CONFIG:-}" ]; then
    echo "To finish, put this in your $config:"
    echo
    [ -n "$path_line" ] && echo "    $path_line"
    echo "    $alias_line"
    exit 0
fi

mkdir -p "$(dirname "$config")"
touch "$config"
added=''
for line in "$path_line" "$alias_line"; do
    if [ -n "$line" ] && ! grep -qxF "$line" "$config"; then
        if [ -z "$added" ]; then
            printf '\n# The Fuck, https://github.com/johannSo/thefuck-rs\n' >> "$config"
            added=1
        fi
        printf '%s\n' "$line" >> "$config"
    fi
done
if [ -n "$added" ]; then
    echo "Added the \`fuck\` alias to $config"
else
    echo "The \`fuck\` alias is already in $config"
fi

# The shell that ran this script can't pick up the alias, a child process
# can't change it. Replace this script with a fresh shell that reads the new
# config, so `fuck` works right away without opening a new terminal.
if [ -z "${THEFUCK_NO_NEW_SHELL:-}" ] && [ -t 1 ] && [ -n "${SHELL:-}" ] \
    && { : </dev/tty; } 2>/dev/null; then
    echo "Starting a fresh $shell_name with it, \`fuck\` works now."
    case "$config" in
        # A login shell, like macOS terminals start, an interactive one
        # doesn't read these.
        */.bash_profile | */.bash_login | */.profile) exec "$SHELL" -l -i </dev/tty ;;
        *) exec "$SHELL" -i </dev/tty ;;
    esac
fi

echo "It works in new shells. To use it in this one run:  . $config"
