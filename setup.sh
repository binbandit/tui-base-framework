#!/usr/bin/env bash
# Turn this template into your own project.
#
# Usage:
#   ./setup.sh <project-name> [options]
#
# Options:
#   --app-only      Fold the framework into a binary-only app: deletes
#                   src/lib.rs and examples/, and makes src/tui/ a module of
#                   the binary. Best when you're building an app, not a lib.
#   --no-examples   Delete the examples/ directory (implied by --app-only).
#   --fresh-git     Start a new git history (deletes .git, makes an initial
#                   commit).
#   --yes           Don't prompt; accept defaults for anything not given as a
#                   flag.
#
# Examples:
#   ./setup.sh my-cool-app                 # rename, keep examples and lib
#   ./setup.sh my-cool-app --app-only      # rename + binary-only app
#
# The script deletes itself after a successful run.

set -euo pipefail

OLD_PKG="tui-base-framework"
OLD_IDENT="tui_base_framework"

err() { printf 'error: %s\n' "$1" >&2; exit 1; }
note() { printf '  %s\n' "$1"; }

cd "$(dirname "$0")"

# ---------------------------------------------------------------------------
# Parse arguments
# ---------------------------------------------------------------------------
NAME="${1:-}"
[ -n "$NAME" ] && [ "${NAME#-}" = "$NAME" ] && shift || NAME=""

APP_ONLY=false
NO_EXAMPLES=false
FRESH_GIT=false
ASSUME_YES=false

while [ $# -gt 0 ]; do
    case "$1" in
        --app-only) APP_ONLY=true; NO_EXAMPLES=true ;;
        --no-examples) NO_EXAMPLES=true ;;
        --fresh-git) FRESH_GIT=true ;;
        --yes|-y) ASSUME_YES=true ;;
        -h|--help) sed -n '2,21p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) err "unknown option: $1 (see ./setup.sh --help)" ;;
    esac
    shift
done

if [ -z "$NAME" ]; then
    $ASSUME_YES && err "a project name is required with --yes"
    printf 'Project name (e.g. my-cool-app): '
    read -r NAME
fi

case "$NAME" in
    ''|*[!a-zA-Z0-9_-]*) err "invalid crate name: '$NAME' (use letters, digits, - and _)" ;;
    [0-9]*) err "crate names cannot start with a digit" ;;
esac
IDENT="$(printf '%s' "$NAME" | tr '-' '_')"

if ! $ASSUME_YES && ! $APP_ONLY; then
    printf 'Fold the framework into a binary-only app (no lib.rs, no examples)? [y/N] '
    read -r reply
    case "$reply" in [yY]*) APP_ONLY=true; NO_EXAMPLES=true ;; esac
fi

echo "Setting up '$NAME'..."

# ---------------------------------------------------------------------------
# Rename the crate everywhere
# ---------------------------------------------------------------------------
find src examples -type f -name '*.rs' \
    -exec sed -i.bak -e "s/$OLD_IDENT/$IDENT/g" -e "s/$OLD_PKG/$NAME/g" {} + 2>/dev/null || true
for f in Cargo.toml Cargo.lock ./*.md examples/*.md; do
    [ -f "$f" ] || continue
    sed -i.bak -e "s/$OLD_IDENT/$IDENT/g" -e "s/$OLD_PKG/$NAME/g" "$f"
done
find src examples . -maxdepth 1 -name '*.bak' -delete 2>/dev/null || true
rm -f examples/*.bak
note "renamed crate to '$NAME' (module path '$IDENT')"

# ---------------------------------------------------------------------------
# Update package metadata
# ---------------------------------------------------------------------------
GIT_NAME="$(git config user.name 2>/dev/null || true)"
GIT_EMAIL="$(git config user.email 2>/dev/null || true)"
if [ -n "$GIT_NAME" ]; then
    AUTHOR="$GIT_NAME${GIT_EMAIL:+ <$GIT_EMAIL>}"
    sed -i.bak "s|^authors = .*|authors = [\"$AUTHOR\"]|" Cargo.toml && rm -f Cargo.toml.bak
    note "set authors to '$AUTHOR' (from git config)"
else
    sed -i.bak '/^authors = /d' Cargo.toml && rm -f Cargo.toml.bak
    note "removed authors (git config has no user.name)"
fi

sed -i.bak \
    -e 's|^description = .*|description = "TODO: describe your app"|' \
    -e '/^repository = /d' \
    -e '/^homepage = /d' \
    -e '/^keywords = /d' \
    -e '/^categories = /d' \
    Cargo.toml && rm -f Cargo.toml.bak
note "reset description; removed repository/homepage/keywords/categories"

# Drop the template-setup comment block and squeeze leftover blank lines.
sed -i.bak '/^# --- template setup /,/^# ----*$/d' Cargo.toml && rm -f Cargo.toml.bak
sed -i.bak '/^$/N;/^\n$/D' Cargo.toml && rm -f Cargo.toml.bak

# ---------------------------------------------------------------------------
# Optional: strip examples
# ---------------------------------------------------------------------------
if $NO_EXAMPLES; then
    rm -rf examples
    # The backticks below are literal doc-comment text, not command expansion.
    # shellcheck disable=SC2016
    sed -i.bak '/from `examples\/` over it/d' src/main.rs && rm -f src/main.rs.bak
    note "removed examples/"
fi

# ---------------------------------------------------------------------------
# Optional: binary-only app (fold the framework into the binary)
# ---------------------------------------------------------------------------
if $APP_ONLY; then
    rm -f src/lib.rs

    # The framework module is self-contained under src/tui/, so the binary
    # adopts it with a `mod tui;` declaration and crate-local imports.
    find src -type f -name '*.rs' -exec sed -i.bak \
        -e "s/use $IDENT::/use crate::tui::/g" \
        -e "s/$IDENT::/crate::tui::/g" {} +
    find src -name '*.bak' -delete
    # In a binary crate, framework API your app doesn't use yet would warn as
    # dead code; allow it on the module until you grow into it.
    sed -i.bak "0,/^use /s//#[allow(dead_code, unused_imports)]\nmod tui;\n\nuse /" src/main.rs \
        && rm -f src/main.rs.bak

    # Drop the now-stale template note from the module docs.
    sed -i.bak '/^\/\/!$/,/binary-only project unchanged/d' src/tui/mod.rs \
        && rm -f src/tui/mod.rs.bak

    # Point doc snippets at the new paths.
    for f in ./*.md; do
        [ -f "$f" ] || continue
        sed -i.bak "s/use $IDENT::/use crate::tui::/g" "$f" && rm -f "$f.bak"
    done
    note "converted to a binary-only app (framework lives in src/tui/)"
fi

# ---------------------------------------------------------------------------
# Optional: fresh git history
# ---------------------------------------------------------------------------
if ! $ASSUME_YES && ! $FRESH_GIT && [ -d .git ]; then
    printf 'Start a fresh git history? [y/N] '
    read -r reply
    case "$reply" in [yY]*) FRESH_GIT=true ;; esac
fi

# ---------------------------------------------------------------------------
# Verify and finish
# ---------------------------------------------------------------------------
rm -f -- "$0"
note "removed setup.sh"

if command -v cargo >/dev/null 2>&1; then
    echo "Verifying with 'cargo check'..."
    cargo fmt --quiet 2>/dev/null || true
    cargo check --all-targets --quiet
    note "cargo check passed"
else
    note "cargo not found; skipping verification"
fi

if $FRESH_GIT; then
    rm -rf .git
    git init -q
    git add -A
    git commit -qm "Initial commit (from tui-base-framework template)"
    note "started fresh git history"
fi

echo
echo "Done. Your app is ready:"
echo "  cargo run"
echo
echo "Start editing src/main.rs."
