#!/usr/bin/env bash
# Fast filesystem regressions. CI separately builds both generated project modes.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/tui-setup-test.XXXXXX")"
trap 'rm -rf "$SCRATCH"' EXIT
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

# Isolate git identity/signing from the developer's machine, and make verification
# failures deterministic without compiling the same crate in every fixture.
export GIT_CONFIG_NOSYSTEM=1
export GIT_CONFIG_GLOBAL="$SCRATCH/gitconfig"
unset GIT_DIR GIT_WORK_TREE GIT_AUTHOR_NAME GIT_AUTHOR_EMAIL GIT_COMMITTER_NAME GIT_COMMITTER_EMAIL
unset GIT_CONFIG_COUNT
: > "$GIT_CONFIG_GLOBAL"
git config --global user.name 'Setup Test'
git config --global user.email 'setup@example.test'
mkdir "$SCRATCH/bin"
cat > "$SCRATCH/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
exit "${SETUP_TEST_CARGO_EXIT:-0}"
CARGO
chmod +x "$SCRATCH/bin/cargo"
export PATH="$SCRATCH/bin:$PATH"

fixture() {
    CASE="$SCRATCH/$1"
    mkdir "$CASE"
    cp "$ROOT/setup.sh" "$ROOT/Cargo.toml" "$ROOT/Cargo.lock" "$CASE/"
    cp -R "$ROOT/src" "$CASE/"
    if [ -d "$ROOT/examples" ]; then cp -R "$ROOT/examples" "$CASE/"; fi
    cp "$ROOT/"*.md "$CASE/"
    if [ -d "$ROOT/.github" ]; then cp -R "$ROOT/.github" "$CASE/"; fi
    mkdir "$CASE/scripts"
    cp "$ROOT/scripts/test-setup.sh" "$CASE/scripts/"
    printf '# template runtime checks\n' > "$CASE/scripts/test-runtime.py"
}
run_setup() {
    (cd "$SCRATCH" && "$CASE/setup.sh" "$@") > "$SCRATCH/output" 2>&1 \
        || { cat "$SCRATCH/output" >&2; fail "setup failed: $*"; }
}
expect_failure() {
    if (cd "$SCRATCH" && "$CASE/setup.sh" "$@") > "$SCRATCH/output" 2>&1; then
        fail "setup unexpectedly succeeded: $*"
    fi
    [ -f "$CASE/setup.sh" ] || fail 'failed setup removed itself'
}
assert_clean() {
    [ ! -f "$CASE/setup.sh" ] || fail 'successful setup did not remove itself'
    [ ! -f "$CASE/scripts/test-setup.sh" ] || fail 'template regression script left behind'
    [ ! -f "$CASE/scripts/test-runtime.py" ] || fail 'template runtime script left behind'
    if grep -E 'setup\.sh|test-setup\.sh|template-only:' "$CASE/.github/workflows/ci.yml"; then
        fail 'generated CI still references template checks'
    fi
    [ -z "$(find "$CASE" -name '*.setup.*' -print)" ] || fail 'temporary files left behind'
    [ -z "$(find "$CASE" -name '*.rs.bak' -print)" ] || fail 'Rust backups left behind'
    if grep -R -E 'tui_base_framework|tui-base-framework' "$CASE/src" "$CASE/Cargo.toml" "$CASE/Cargo.lock"; then
        fail 'stale crate names'
    fi
}

fixture 'rename with spaces'
printf 'keep me\n' > "$CASE/notes.bak"
printf 'keep me too\n' > "$CASE/src/tui/user.bak"
run_setup my-test-app --yes
assert_clean
[ -f "$CASE/src/lib.rs" ] && [ -d "$CASE/examples" ] || fail 'rename removed library/examples'
[ "$(cat "$CASE/notes.bak")" = 'keep me' ] || fail 'deleted unrelated root backup'
[ "$(cat "$CASE/src/tui/user.bak")" = 'keep me too' ] || fail 'deleted unrelated nested backup'

fixture app-only
run_setup my-test-app --app-only --yes
assert_clean
[ ! -f "$CASE/src/lib.rs" ] && [ ! -d "$CASE/examples" ] || fail 'app-only kept library/examples'
[ "$(grep -c '^mod tui;' "$CASE/src/main.rs")" = 1 ] || fail 'module declaration missing/duplicated'
if grep -R 'my_test_app::' "$CASE/src" "$CASE/"*.md; then fail 'stale app-only import paths'; fi

fixture no-examples
run_setup my-test-app --no-examples --yes
assert_clean
[ -f "$CASE/src/lib.rs" ] && [ ! -d "$CASE/examples" ] || fail 'no-examples removed wrong files'

fixture invalid-names
cp "$CASE/Cargo.toml" "$SCRATCH/original.toml"
for name in 123bad 'has space' _ self Self async gen tui tokio signal-hook; do
    expect_failure "$name" --yes
    cmp "$SCRATCH/original.toml" "$CASE/Cargo.toml" || fail "invalid name mutated manifest: $name"
done

fixture missing-import
printf 'fn main() {}\n' > "$CASE/src/main.rs"
expect_failure my-test-app --app-only --yes
cmp "$SCRATCH/original.toml" "$CASE/Cargo.toml" || fail 'missing import mutated manifest'
[ -f "$CASE/src/lib.rs" ] || fail 'missing import removed library'

fixture author-escaping
git config --global user.name $'A & B | "Quoted" \\ Team\tName\nNext'
run_setup my-test-app --yes
expected='authors = ["A & B | \"Quoted\" \\ Team\u0009Name\u000aNext <setup@example.test>"]'
grep -Fqx "$expected" "$CASE/Cargo.toml" || fail 'author metadata was not escaped literally'
git config --global user.name 'Setup Test'

fixture retry
SETUP_TEST_CARGO_EXIT=1 expect_failure my-test-app --app-only --yes
expect_failure different-name --app-only --yes
run_setup my-test-app --app-only --yes
assert_clean
[ "$(grep -c '^mod tui;' "$CASE/src/main.rs")" = 1 ] || fail 'retry duplicated module declaration'

fixture retry-containing-template-name
SETUP_TEST_CARGO_EXIT=1 expect_failure tui-base-framework-app --yes
run_setup tui-base-framework-app --yes
grep -q '^name = "tui-base-framework-app"$' "$CASE/Cargo.toml" || fail 'retry expanded the new name'
grep -q 'use tui_base_framework_app::' "$CASE/src/main.rs" || fail 'retry expanded imports'

fixture linked-worktree
printf 'gitdir: /some/other/repo\n' > "$CASE/.git"
expect_failure my-test-app --fresh-git --yes
cmp "$SCRATCH/original.toml" "$CASE/Cargo.toml" || fail 'worktree rejection mutated manifest'

fixture fresh-git
# Local-only identity must survive replacement of the old repository.
git -C "$CASE" init -q
git -C "$CASE" config user.name 'Local Author'
git -C "$CASE" config user.email 'local@example.test'
git -C "$CASE" add -A
git -C "$CASE" commit -qm 'Old history'
old_head="$(git -C "$CASE" rev-parse HEAD)"
# A failed verification must preserve the original history and setup script.
SETUP_TEST_CARGO_EXIT=1 expect_failure my-test-app --fresh-git --yes
[ "$(git -C "$CASE" rev-parse HEAD)" = "$old_head" ] || fail 'failed check replaced git history'
# Signing failure happens after verification, but must also leave .git intact.
git config --global commit.gpgsign true
git config --global gpg.program false
git config --global user.signingkey 'setup-test-key'
expect_failure my-test-app --fresh-git --yes
[ "$(git -C "$CASE" rev-parse HEAD)" = "$old_head" ] || fail 'failed commit replaced git history'
git config --global --unset commit.gpgsign
git config --global --unset gpg.program
git config --global --unset user.signingkey
run_setup my-test-app --fresh-git --yes
assert_clean
[ "$(git -C "$CASE" rev-list --count HEAD)" = 1 ] || fail 'fresh git retained old history'
[ "$(git -C "$CASE" log -1 --format=%an)" = 'Local Author' ] || fail 'fresh git lost local identity'
[ -z "$(git -C "$CASE" status --porcelain)" ] || fail 'fresh git left uncommitted changes'

# Exercise the relative-path entry point, which changes directory internally.
fixture relative-help
(cd "$SCRATCH" && ./relative-help/setup.sh --help) > "$SCRATCH/output"
grep -q 'Usage:' "$SCRATCH/output" || fail 'help failed from another directory'
(cd "$SCRATCH" && ./relative-help/setup.sh my-test-app --yes) > "$SCRATCH/output"
assert_clean

printf 'Setup regression tests passed.\n'
