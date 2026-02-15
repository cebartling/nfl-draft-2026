---
name: github-comments-resolve
description: Fetch all open PR review comments, prioritize by severity, fix critical and high issues first with individual commits, reply to each resolved comment in-thread, run tests, push, and summarize remaining items. Use when the user wants to systematically resolve PR review feedback.
---

# GitHub Comments Resolve

A skill for systematically resolving PR review comments by severity. Fetches all open review comments, classifies them, fixes the most important ones first, replies in-thread to confirm each resolution, and verifies everything with tests.

## When to Use

- The user says "resolve review comments", "fix PR feedback", "address comments", or "clean up the PR"
- A PR has accumulated review feedback that needs to be worked through methodically
- The user wants a severity-driven approach rather than fixing comments in arbitrary order

## Prerequisites

- **`gh` CLI** installed and authenticated
- An open PR exists on the current branch
- Working tree is clean (no uncommitted changes) — if dirty, ask the user to commit or stash first

## Workflow

### Step 1: Fetch All Open Review Comments

Get the PR number, owner, and repo:

```bash
PR_NUMBER=$(gh pr view --json number --jq '.number')
REPO_INFO=$(gh repo view --json owner,name --jq '"\(.owner.login)/\(.name)"')
OWNER=$(echo "$REPO_INFO" | cut -d'/' -f1)
REPO=$(echo "$REPO_INFO" | cut -d'/' -f2)
```

Fetch all review comments on the PR:

```bash
gh api repos/{owner}/{repo}/pulls/{pr_number}/comments
```

This returns inline review comments — the ones attached to specific lines of code. For each comment, extract:

- `id` — the comment ID (needed for threaded replies)
- `path` — the file the comment is on
- `line` (or `original_line`) — the line number
- `body` — the reviewer's feedback
- `user.login` — who left the comment
- `created_at` — when it was posted
- `in_reply_to_id` — if set, this comment is itself a reply (skip these as they're part of an existing thread)

**Filter out already-resolved comments.** A comment is considered resolved if:
- A reply in its thread contains a message starting with "Fixed:" (from a previous run of this skill)
- The comment thread has been explicitly resolved in the GitHub UI

To check for existing replies, look for comments where `in_reply_to_id` matches the parent comment's `id`. If any reply starts with "Fixed:", treat the parent as resolved.

### Step 2: Classify Comments by Severity

Read each unresolved comment and classify it into one of four severity levels based on its content:

| Severity | Criteria | Examples |
|---|---|---|
| **CRITICAL** | Security vulnerabilities, data loss risks, crashes, broken functionality | SQL injection, unhandled null causing panic, auth bypass, data corruption |
| **HIGH** | Bugs, incorrect logic, missing error handling, performance issues with real impact | Wrong return type, missing validation, unbounded query, race condition |
| **MEDIUM** | Code quality, maintainability, minor improvements, style violations with substance | Missing docs on public API, complex function that should be split, unclear naming |
| **LOW** | Nitpicks, cosmetic preferences, optional suggestions | Whitespace, import ordering, "you could also do X", minor naming preference |

Present the classified list to the user:

```
## Review Comments by Severity

### CRITICAL (1)
1. [#42] auth.rs:87 — "This directly interpolates user input into the SQL query" (@reviewer)

### HIGH (3)
2. [#38] handler.rs:42 — "unwrap() will panic if the token is missing" (@copilot)
3. [#41] api.rs:118 — "No validation on the email field" (@reviewer)
4. [#45] db.rs:203 — "This query has no LIMIT — could return millions of rows" (@copilot)

### MEDIUM (2)
5. [#39] models.rs:15 — "Add doc comments to these public structs" (@reviewer)
6. [#44] utils.rs:67 — "This function does too many things, consider splitting" (@reviewer)

### LOW (1)
7. [#40] config.rs:3 — "Unused import" (@copilot)
```

The `[#id]` is the comment ID needed for replies in Step 4.

### Step 3: Fix CRITICAL and HIGH Issues

Work through CRITICAL comments first, then HIGH. For each comment:

1. **Read the surrounding code** — understand the full context, not just the flagged line
2. **Determine the correct fix** — address the reviewer's actual concern
3. **Apply the change** — edit the file
4. **Commit individually** — one commit per logical fix

```bash
git add <file>
git commit -m "fix(<scope>): <what was fixed>

Resolves review comment #<comment_id>: <brief description>"
```

**One commit per logical fix**, not necessarily one commit per comment. If two comments point to the same root cause (e.g., two places where the same validation is missing), fix them together in a single commit and reply to both.

If a CRITICAL or HIGH comment is ambiguous or you disagree with the assessment, flag it for the user rather than guessing at a fix.

### Step 4: Reply to Each Resolved Comment

After fixing a comment, reply **in-thread** using the `in_reply_to` parameter to maintain proper threading:

```bash
gh api repos/{owner}/{repo}/pulls/{pr_number}/comments \
  -method POST \
  -f body="Fixed: <description of what was changed>" \
  -F in_reply_to=<comment_id>
```

**IMPORTANT:** Use the pull request review comments endpoint with `in_reply_to` for threading. Do NOT use the `issues/{number}/comments` endpoint — that creates top-level PR comments instead of threaded replies.

The reply body should be specific about what changed:

- ✅ `"Fixed: Replaced string interpolation with parameterized query to prevent SQL injection"`
- ✅ `"Fixed: Added .ok_or() with descriptive error instead of unwrap()"`
- ❌ `"Fixed"` (too vague)
- ❌ `"Done"` (not helpful to the reviewer)

### Step 5: Run Full Test Suite

After all CRITICAL and HIGH fixes are applied, run the project's tests.

**Backend (Rust/Cargo):**

```bash
cargo test --workspace
```

**Frontend (from package.json):**

```bash
# Detect and run the appropriate test command
npm test
# or: yarn test / pnpm test
```

If tests fail:
- Read the failure output
- Determine if the failure is related to a fix you just made
- If yes, fix it and amend the relevant commit or create a follow-up commit
- If no (pre-existing failure), note it in the summary but don't block on it
- Re-run tests until the suite passes or only pre-existing failures remain

### Step 6: Push and Summarize

Push all fix commits:

```bash
git push origin HEAD
```

Provide a complete summary:

```
## Review Comments Resolution Summary

### Resolved (4)
- ✅ CRITICAL [#42] auth.rs:87 — Parameterized SQL query (commit abc1234)
- ✅ HIGH [#38] handler.rs:42 — Replaced unwrap with proper error handling (commit def5678)
- ✅ HIGH [#41] api.rs:118 — Added email format validation (commit ghi9012)
- ✅ HIGH [#45] db.rs:203 — Added LIMIT clause to query (commit jkl3456)

### Remaining Unresolved (3)
- ⏳ MEDIUM [#39] models.rs:15 — "Add doc comments to public structs"
- ⏳ MEDIUM [#44] utils.rs:67 — "Split this function"
- ⏳ LOW [#40] config.rs:3 — "Unused import"

### Test Results
- Backend (cargo test): ✅ 142 passed, 0 failed
- Frontend (npm test): ✅ 87 passed, 0 failed

### Commits Pushed (4)
- fix(auth): parameterize SQL query to prevent injection
- fix(handler): return Result instead of panicking on missing token
- fix(api): validate email format before processing
- fix(db): add LIMIT to unbounded user query
```

If MEDIUM or LOW items are quick wins (like removing an unused import), mention that to the user and offer to fix them in a follow-up pass.

## Edge Cases

**No review comments:** If the API returns no comments or all are already resolved, report that the PR is clean and skip to tests.

**Comment on deleted code:** If a comment references lines that no longer exist in the current branch, note it as stale and skip it. Reply to the comment explaining it's no longer applicable.

**Ambiguous severity:** When in doubt, classify one level higher. It's better to fix a MEDIUM-classified-as-HIGH than to skip a HIGH-classified-as-MEDIUM.

**Rate limits:** The `gh api` calls are subject to GitHub's rate limits. If you hit a 403, wait and retry. For PRs with many comments, batch the reply calls rather than firing them all at once.

**Bot-generated comments:** Treat Copilot and other bot comments the same as human comments — they still need to be resolved. Classify them by the same severity criteria.
