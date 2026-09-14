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

# Resolve the script before changing directories: callers may run it from elsewhere.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"
SCRIPT="$SCRIPT_DIR/$(basename "$0")"

# Replace only the file being edited. Backup-suffix editing left nested backups
# behind and its cleanup could delete a user's unrelated .bak files.
rewrite() {
    local file="$1" temporary
    shift
    temporary="$(mktemp "${file}.setup.XXXXXX")"
    cp -p "$file" "$temporary"
    if "$@" "$file" > "$temporary"; then
        mv "$temporary" "$file"
    else
        rm -f "$temporary"
        return 1
    fi
}

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
        -h|--help) sed -n '2,21p' "$SCRIPT" | sed 's/^# \{0,1\}//'; exit 0 ;;
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
    tui) err "'tui' collides with the framework's internal module name; pick another" ;;
esac
IDENT="$(printf '%s' "$NAME" | tr '-' '_')"
# These names either cannot appear in a Rust 2024 use path or shadow a crate
# imported by the template. Check before rewriting any files.
case "$IDENT" in
    _|as|async|await|break|const|continue|crate|dyn|else|enum|extern|false|fn|for|if|impl|in|let|loop|match|mod|move|mut|pub|ref|return|self|Self|static|struct|super|trait|true|type|unsafe|use|where|while|abstract|become|box|do|final|gen|macro|override|priv|try|typeof|unsized|virtual|yield)
        err "'$NAME' is a reserved Rust identifier; pick another" ;;
    std|core|alloc|anyhow|crossterm|ratatui|tokio|signal_hook)
        err "'$NAME' collides with a crate used by the template; pick another" ;;
esac
[ -f Cargo.toml ] && [ -f src/main.rs ] && [ -d src/tui ] \
    || err "run setup from a complete template checkout"
grep -Eq "^name = \"($OLD_PKG|$NAME)\"$" Cargo.toml \
    || err "this project was already renamed; retry with its current name"

if ! $ASSUME_YES && ! $APP_ONLY; then
    printf 'Fold the framework into a binary-only app (no lib.rs, no examples)? [y/N] '
    read -r reply
    case "$reply" in [yY]*) APP_ONLY=true; NO_EXAMPLES=true ;; esac
fi

if $APP_ONLY; then
    grep -q '^use ' src/main.rs \
        || err "cannot insert 'mod tui;' into src/main.rs (no top-level 'use' line)"
fi

# Ask and validate before mutating the project. A worktree/submodule has a .git
# file pointing elsewhere; replacing it would detach it from its parent repo.
if ! $ASSUME_YES && ! $FRESH_GIT && [ -d .git ]; then
    printf 'Start a fresh git history? [y/N] '
    read -r reply
    case "$reply" in [yY]*) FRESH_GIT=true ;; esac
fi
if $FRESH_GIT; then
    command -v git >/dev/null 2>&1 || err "--fresh-git requires git"
    [ ! -f .git ] && [ ! -L .git ] \
        || err "--fresh-git cannot be used in a linked worktree or submodule"
    if ! git var GIT_AUTHOR_IDENT >/dev/null 2>&1 \
        || ! git var GIT_COMMITTER_IDENT >/dev/null 2>&1; then
        err "--fresh-git requires a configured git author and committer"
    fi
fi

echo "Setting up '$NAME'..."

# ---------------------------------------------------------------------------
# Rename the crate everywhere
# ---------------------------------------------------------------------------
# A retry must not expand a new name that itself contains the template name.
if grep -q "^name = \"$OLD_PKG\"$" Cargo.toml; then
    SOURCE_DIRS=(src)
    [ ! -d examples ] || SOURCE_DIRS+=(examples)
    while IFS= read -r -d '' file; do
        rewrite "$file" sed -e "s/$OLD_IDENT/$IDENT/g" -e "s/$OLD_PKG/$NAME/g"
    done < <(find "${SOURCE_DIRS[@]}" -type f -name '*.rs' -print0)
    for file in Cargo.toml Cargo.lock ./*.md examples/*.md; do
        [ -f "$file" ] || continue
        rewrite "$file" sed -e "s/$OLD_IDENT/$IDENT/g" -e "s/$OLD_PKG/$NAME/g"
    done
fi
note "renamed crate to '$NAME' (module path '$IDENT')"

# ---------------------------------------------------------------------------
# Update package metadata
# ---------------------------------------------------------------------------
GIT_NAME="$(git config user.name 2>/dev/null || true)"
GIT_EMAIL="$(git config user.email 2>/dev/null || true)"
AUTHOR="${GIT_NAME:+$GIT_NAME${GIT_EMAIL:+ <$GIT_EMAIL>}}"
# ENVIRON preserves literal backslashes (awk -v and sed replacements do not).
# Encode all TOML basic-string escapes, including control characters.
AUTHOR="$AUTHOR" rewrite Cargo.toml awk '
    function quoted(value,    i, c, n) {
        printf "\""
        for (i = 1; i <= length(value); i++) {
            c = substr(value, i, 1)
            if (c == "\\" || c == "\"") printf "\\%s", c
            else {
                for (n = 1; n < 32; n++) if (c == sprintf("%c", n)) break
                if (n < 32 || c == sprintf("%c", 127)) printf "\\u%04x", (n < 32 ? n : 127)
                else printf "%s", c
            }
        }
        printf "\""
    }
    /^authors = / {
        if (ENVIRON["AUTHOR"] != "") {
            printf "authors = ["
            quoted(ENVIRON["AUTHOR"])
            print "]"
        }
        next
    }
    { print }
'
if [ -n "$AUTHOR" ]; then
    note "set authors from git config"
else
    note "removed authors (git config has no user.name)"
fi

rewrite Cargo.toml sed \
    -e 's|^description = .*|description = "TODO: describe your app"|' \
    -e '/^repository = /d' \
    -e '/^homepage = /d' \
    -e '/^keywords = /d' \
    -e '/^categories = /d'
note "reset description; removed repository/homepage/keywords/categories"

# Drop the template-setup comment block and squeeze leftover blank lines.
rewrite Cargo.toml sed '/^# --- template setup /,/^# ----*$/d'
rewrite Cargo.toml awk 'NF { blank = 0; print; next } !blank++ { print }'

# Strip template-maintenance sections from the agent docs; the framework guide
# in the rest of the file still applies to the generated app.
if [ -f AGENTS.md ]; then
    rewrite AGENTS.md sed '/<!-- template-only:start -->/,/<!-- template-only:end -->/d'
    rewrite AGENTS.md awk 'NF { blank = 0; print; next } !blank++ { print }'
    note "trimmed AGENTS.md to the app-facing guide"
fi

# ---------------------------------------------------------------------------
# Optional: strip examples
# ---------------------------------------------------------------------------
if $NO_EXAMPLES; then
    rm -rf examples
    # The backticks below are literal doc-comment text, not command expansion.
    # shellcheck disable=SC2016
    rewrite src/main.rs sed '/from `examples\/` over it/d'
    note "removed examples/"
fi

# ---------------------------------------------------------------------------
# Optional: binary-only app (fold the framework into the binary)
# ---------------------------------------------------------------------------
if $APP_ONLY; then
    rm -f src/lib.rs

    # The framework module is self-contained under src/tui/, so the binary
    # adopts it with a `mod tui;` declaration and crate-local imports.
    while IFS= read -r -d '' file; do
        rewrite "$file" sed "s/$IDENT::/crate::tui::/g"
    done < <(find src -type f -name '*.rs' -print0)
    # Unused framework API stays available as the app grows. Avoid adding the
    # module twice when retrying after a failed cargo check.
    if ! grep -q '^mod tui;' src/main.rs; then
        rewrite src/main.rs awk '!done && /^use / {
                 print "#[allow(dead_code, unused_imports)]"
                 print "mod tui;"
                 print ""
                 done = 1
             }
             { print }'
    fi

    # Drop the now-stale template note from the module docs.
    rewrite src/tui/mod.rs sed '/^\/\/! This folder is deliberately self-contained/,/binary-only project unchanged/d'

    # Point doc snippets at the new paths.
    for f in ./*.md; do
        [ -f "$f" ] || continue
        rewrite "$f" sed "s/$IDENT::/crate::tui::/g"
    done
    note "converted to a binary-only app (framework lives in src/tui/)"
fi

# CI for a generated app must not try to run the setup script after it deletes
# itself. Keep the ordinary Rust checks and remove only marked template checks.
if [ -f .github/workflows/ci.yml ]; then
    rewrite .github/workflows/ci.yml sed '/# template-only:start/,/# template-only:end/d'
fi
rm -f scripts/test-setup.sh scripts/test-runtime.py
if [ -d scripts ] && [ -z "$(ls -A scripts)" ]; then rmdir scripts; fi

# ---------------------------------------------------------------------------
# Verify and finish
# ---------------------------------------------------------------------------
if command -v cargo >/dev/null 2>&1; then
    echo "Verifying with 'cargo check'..."
    cargo fmt --all --quiet
    cargo check --all-targets --quiet
    note "cargo check passed"
else
    note "cargo not found; skipping verification"
fi

if $FRESH_GIT; then
    # Build the replacement history separately. Keep the original .git intact
    # until the initial commit succeeds (identity, hooks, and signing can fail).
    NEW_GIT="$(mktemp -d "${TMPDIR:-/tmp}/tui-setup-git.XXXXXX")"
    trap 'rm -rf "$NEW_GIT"' EXIT
    git init -q --separate-git-dir="$NEW_GIT/history" "$NEW_GIT/worktree"
    if [ -n "$GIT_NAME" ]; then
        git --git-dir="$NEW_GIT/history" config user.name "$GIT_NAME"
    fi
    if [ -n "$GIT_EMAIL" ]; then
        git --git-dir="$NEW_GIT/history" config user.email "$GIT_EMAIL"
    fi
    git --git-dir="$NEW_GIT/history" --work-tree="$SCRIPT_DIR" add -A
    git --git-dir="$NEW_GIT/history" --work-tree="$SCRIPT_DIR" rm --cached --ignore-unmatch -- "$(basename "$SCRIPT")"
    git --git-dir="$NEW_GIT/history" --work-tree="$SCRIPT_DIR" commit -qm "Initial commit (from tui-base-framework template)"
    rm -rf .git
    mv "$NEW_GIT/history" .git
    rm -rf "$NEW_GIT"
    trap - EXIT
    note "started fresh git history"
fi

# A failed verification keeps the script available for a same-name retry.
rm -f -- "$SCRIPT"
note "removed setup.sh"

echo
echo "Done. Your app is ready:"
echo "  cargo run"
echo
echo "Start editing src/main.rs. CHEATSHEET.md has copy-paste patterns for"
echo "input, layouts, widgets, async work, and tests."
