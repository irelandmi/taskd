#!/usr/bin/env bash
set -euo pipefail

# End-to-end CLI test against a temp database.
# Usage: ./tests/cli_e2e.sh [path-to-taskd-binary]
#
# Builds the binary if no path is given.

TASKD="${1:-}"
if [ -z "$TASKD" ]; then
	echo "==> Building taskd..."
	cargo build --bin taskd --quiet
	TASKD="$(cargo metadata --format-version 1 --no-deps | grep -o '"target_directory":"[^"]*"' | cut -d'"' -f4)/debug/taskd"
fi

DB="$(mktemp /tmp/taskd-test-XXXXXX.db)"
trap 'rm -f "$DB"' EXIT

run() {
	"$TASKD" --db "$DB" "$@"
}

assert_contains() {
	local output="$1" pattern="$2" msg="${3:-}"
	if ! echo "$output" | grep -q "$pattern"; then
		echo "FAIL: expected output to contain '$pattern' ${msg:+(${msg})}"
		echo "  got: $output"
		exit 1
	fi
}

assert_not_contains() {
	local output="$1" pattern="$2" msg="${3:-}"
	if echo "$output" | grep -q "$pattern"; then
		echo "FAIL: expected output NOT to contain '$pattern' ${msg:+(${msg})}"
		echo "  got: $output"
		exit 1
	fi
}

assert_fails() {
	if "$@" 2>/dev/null; then
		echo "FAIL: expected command to fail: $*"
		exit 1
	fi
}

extract_id() {
	# Extract the human-readable ID from output like "created project Foo (bold-fox-a3f1)"
	echo "$1" | grep -o '([^ ]*)' | tr -d '()'
}

PASS=0
fail_count=0

pass() {
	PASS=$((PASS + 1))
	echo "  PASS: $1"
}

# ============================================================
echo "--- Project CRUD ---"

OUT=$(run project create "Test Project" --description "A test")
assert_contains "$OUT" "created project Test Project"
PROJECT_ID=$(extract_id "$OUT")
pass "create project"

OUT=$(run project list)
assert_contains "$OUT" "Test Project"
pass "list projects"

OUT=$(run project show "$PROJECT_ID")
assert_contains "$OUT" "Test Project"
assert_contains "$OUT" "A test"
pass "show project"

# Backlog epic auto-created
OUT=$(run epic list --project "$PROJECT_ID")
assert_contains "$OUT" "Backlog"
pass "backlog epic auto-created"

# ============================================================
echo "--- Epic CRUD ---"

OUT=$(run epic create --project "$PROJECT_ID" "Sprint 1" --description "First sprint")
assert_contains "$OUT" "created epic Sprint 1"
EPIC_ID=$(extract_id "$OUT")
pass "create epic"

OUT=$(run epic show "$EPIC_ID")
assert_contains "$OUT" "Sprint 1"
assert_contains "$OUT" "open"
pass "show epic"

OUT=$(run epic close "$EPIC_ID")
assert_contains "$OUT" "closed"
OUT=$(run epic show "$EPIC_ID")
assert_contains "$OUT" "closed"
pass "close epic"

# Reopen by listing — it should still be there
OUT=$(run epic list --project "$PROJECT_ID")
assert_contains "$OUT" "Sprint 1"
assert_contains "$OUT" "Backlog"
pass "list epics"

# ============================================================
echo "--- Task CRUD (with explicit epic) ---"

OUT=$(run task create --project "$PROJECT_ID" --epic "$EPIC_ID" "Build login" --kind story --priority high --assignee alice)
assert_contains "$OUT" "created story Build login"
TASK_ID=$(extract_id "$OUT")
pass "create task with epic"

OUT=$(run task show "$TASK_ID")
assert_contains "$OUT" "Build login"
assert_contains "$OUT" "story"
assert_contains "$OUT" "high"
assert_contains "$OUT" "alice"
pass "show task"

OUT=$(run task update "$TASK_ID" --status in_progress --title "Build login page")
assert_contains "$OUT" "Build login page"
pass "update task"

OUT=$(run task list --project "$PROJECT_ID")
assert_contains "$OUT" "Build login page"
assert_contains "$OUT" "in_progress"
pass "list tasks"

# ============================================================
echo "--- Task without epic (backlog) ---"

OUT=$(run task create --project "$PROJECT_ID" "Quick fix typo")
assert_contains "$OUT" "created task Quick fix typo"
QUICK_ID=$(extract_id "$OUT")
pass "create task without epic"

OUT=$(run task show "$QUICK_ID")
assert_contains "$OUT" "Quick fix typo"
pass "backlog task show"

# ============================================================
echo "--- Sub-tasks ---"

OUT=$(run task create --project "$PROJECT_ID" --epic "$EPIC_ID" --parent "$TASK_ID" "Build form component")
assert_contains "$OUT" "created task Build form component"
CHILD_ID=$(extract_id "$OUT")
pass "create sub-task"

OUT=$(run task show "$TASK_ID")
assert_contains "$OUT" "children:"
assert_contains "$OUT" "Build form component"
pass "parent shows children"

OUT=$(run task show "$CHILD_ID")
assert_contains "$OUT" "parent:"
pass "child shows parent"

# ============================================================
echo "--- Task filters ---"

OUT=$(run task list --project "$PROJECT_ID" --status in_progress)
assert_contains "$OUT" "Build login page"
assert_not_contains "$OUT" "Quick fix typo"
pass "filter by status"

OUT=$(run task list --project "$PROJECT_ID" --kind story)
assert_contains "$OUT" "Build login page"
assert_not_contains "$OUT" "Quick fix typo"
pass "filter by kind"

OUT=$(run task list --project "$PROJECT_ID" --assignee alice)
assert_contains "$OUT" "Build login page"
assert_not_contains "$OUT" "Quick fix typo"
pass "filter by assignee"

# ============================================================
echo "--- Labels ---"

OUT=$(run label create "bug" --color "#ff0000")
assert_contains "$OUT" "created label bug"
LABEL_ID=$(extract_id "$OUT")
pass "create label"

OUT=$(run label list)
assert_contains "$OUT" "bug"
assert_contains "$OUT" "#ff0000"
pass "list labels"

OUT=$(run task create --project "$PROJECT_ID" "Fix crash" --kind bug --label bug)
assert_contains "$OUT" "created bug Fix crash"
BUG_ID=$(extract_id "$OUT")
pass "create task with label"

OUT=$(run task show "$BUG_ID")
assert_contains "$OUT" "labels:.*bug"
pass "task shows labels"

OUT=$(run task list --project "$PROJECT_ID" --label bug)
assert_contains "$OUT" "Fix crash"
assert_not_contains "$OUT" "Build login"
pass "filter by label"

# ============================================================
echo "--- Task done ---"

OUT=$(run task done "$BUG_ID")
assert_contains "$OUT" "done"
OUT=$(run task show "$BUG_ID")
assert_contains "$OUT" "done"
pass "mark task done"

# ============================================================
echo "--- Deletions ---"

run task delete "$CHILD_ID"
OUT=$(run task show "$TASK_ID")
assert_not_contains "$OUT" "children:"
pass "delete sub-task"

run task delete "$BUG_ID"
OUT=$(run task list --project "$PROJECT_ID")
assert_not_contains "$OUT" "Fix crash"
pass "delete task"

run label delete "$LABEL_ID"
OUT=$(run label list)
assert_not_contains "$OUT" "bug"
pass "delete label"

run epic delete "$EPIC_ID"
OUT=$(run epic list --project "$PROJECT_ID")
assert_not_contains "$OUT" "Sprint 1"
pass "delete epic cascades"

# Task that was in the deleted epic should be gone
OUT=$(run task list --project "$PROJECT_ID")
assert_not_contains "$OUT" "Build login page"
pass "epic cascade deleted tasks"

run project delete "$PROJECT_ID"
OUT=$(run project list)
assert_not_contains "$OUT" "Test Project"
pass "delete project"

# ============================================================
echo "--- Error cases ---"

assert_fails run project show "nonexistent-id-0000"
pass "show nonexistent project fails"

assert_fails run task create --project "nonexistent-id-0000" "orphan"
pass "create task for nonexistent project fails"

# Create two projects to test ambiguous prefix
OUT=$(run project create "Proj A")
ID_A=$(extract_id "$OUT")
OUT=$(run project create "Proj B")
ID_B=$(extract_id "$OUT")

# Full IDs should work
run project show "$ID_A" >/dev/null
run project show "$ID_B" >/dev/null
pass "exact ID lookup works"

# ============================================================
echo ""
echo "=== All $PASS tests passed ==="
