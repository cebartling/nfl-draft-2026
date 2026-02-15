---
name: pr-cycle
description: Run a full PR cycle — commit staged changes, push, update the PR description, check for review comments, fix each issue with individual commits, re-run tests, and summarize results. Use when the user wants to wrap up work on a PR, address review feedback, or do a complete commit-push-review-fix pass.
---

# PR Cycle

A skill for running a complete pull request cycle: commit, push, update the PR, process review feedback, fix issues, and verify with tests. Designed for projects using GitHub PRs with `gh` CLI.

## When to Use

- The user says "pr cycle", "push and update the PR", "address review comments", or "wrap up this PR"
- Staged changes are ready to commit and the user wants the full loop handled
- Review feedback has come in and needs to be resolved systematically

## Prerequisites

- **`gh` CLI** must be installed and authenticated
- An open PR must exist for the current branch (or the user intends to create one separately)
- Git working directory should be in a clean-enough state with staged changes ready to commit

## Workflow

### Step 1: Commit Staged Changes

Review what's staged and craft a conventional commit message:

```bash
git diff --cached --stat
```

Write a commit message following the **Conventional Commits** format:

```
<type>(<scope>): <short summary>

<optional body explaining what changed and why>
```

Common types: `feat`, `fix`, `refactor`, `docs`, `style`, `test`, `chore`, `perf`, `ci`

Choose the type and scope based on the actual staged changes — read the diffs to understand what changed, don't guess. If the staged changes span multiple concerns, ask the user whether to commit them together or split them up.

```bash
git commit -m "<type>(<scope>): <summary>"
```

### Step 2: Push to Current Branch

```bash
git push origin HEAD
```

If the push is rejected due to remote changes, pull with rebase first:

```bash
git pull --rebase origin $(git branch --show-current)
git push origin HEAD
```

### Step 3: Update the PR Description

Read the current PR body and the full diff to write an accurate, up-to-date description:

```bash
gh pr view --json body,title
gh pr diff
```

Update the PR description to reflect the current state of all changes in the branch — not just the latest commit. The description should give a reviewer a clear picture of what the PR does overall.

```bash
gh pr edit --body "<updated description>"
```

Keep the description concise and structured. A good format:

```markdown
## What

<Brief summary of what this PR does>

## Changes

- <Key change 1>
- <Key change 2>

## Testing

<How it was tested>
```

### Step 4: Check for Review Comments

Fetch Copilot and human review comments:

```bash
gh pr checks
gh pr view --json reviews,comments
```

Also fetch inline review comments on specific files:

```bash
gh api repos/{owner}/{repo}/pulls/{pr_number}/comments
```

To get the owner, repo, and PR number:

```bash
gh pr view --json number,url --jq '.number'
gh repo view --json owner,name --jq '"\(.owner.login)/\(.name)"'
```

Parse the response to identify all review comments, noting:
- File path and line number
- Comment body (the actual feedback)
- Author
- Whether it's resolved or unresolved

### Step 5: List All Unresolved Review Comments

Present a numbered summary of every unresolved comment:

```
1. [file.rs:42] — "This unwrap could panic on invalid input" (copilot)
2. [api.rs:118] — "Consider returning a Result here instead" (reviewer-name)
3. [component.tsx:27] — "Missing key prop in list rendering" (copilot)
```

If there are no unresolved comments, skip ahead to Step 8 (re-run tests).

### Step 6: Fix Each Issue Individually

For each unresolved comment:

1. **Read the relevant code** — understand the context around the flagged line
2. **Determine the fix** — address the reviewer's concern directly
3. **Apply the change** — edit the file
4. **Commit with a descriptive message** — one commit per fix so the reviewer can trace each resolution

```bash
git add <file>
git commit -m "fix(<scope>): <what was fixed>

Addresses review comment: <brief description of the feedback>"
```

Do NOT batch multiple review fixes into a single commit. Each fix gets its own commit so reviewers can see exactly what changed in response to each comment.

If a comment requires clarification or is a matter of opinion rather than a bug, flag it for the user instead of making a change.

### Step 7: Push All Fixes

```bash
git push origin HEAD
```

### Step 8: Re-run Tests

Run the project test suites to verify nothing is broken.

**Backend (Rust/Cargo):**

```bash
cargo test --workspace
```

**Frontend (from package.json):**

Detect the test command from `package.json` and run it:

```bash
# Check what test script is available
cat package.json | grep -A2 '"test"'

# Run it (typically one of these)
npm test
# or
yarn test
# or
pnpm test
```

If either suite fails:
- Read the failure output carefully
- Fix the broken test or the code causing the failure
- Commit the fix: `fix(<scope>): resolve failing test — <brief reason>`
- Re-run tests to confirm they pass
- Push again

### Step 9: Summarize

Provide a clear summary of everything that happened:

```
## PR Cycle Summary

**Commits made:**
- feat(auth): add session timeout handling
- fix(auth): handle missing token edge case (review feedback)
- fix(ui): add key prop to session list (review feedback)

**Review comments addressed:** 3/3 resolved
- ✅ [auth.rs:42] — Added proper error handling for missing token
- ✅ [auth.rs:118] — Changed unwrap to Result return
- ✅ [SessionList.tsx:27] — Added unique key prop

**Test results:**
- Backend (cargo test): ✅ 142 passed, 0 failed
- Frontend (npm test): ✅ 87 passed, 0 failed

**PR updated:** Description reflects all current changes
```

If any review comments were skipped or flagged for discussion, call them out explicitly so the user can follow up.

## Edge Cases

**No staged changes:** If nothing is staged when the cycle starts, check for unstaged changes and ask the user what to stage. Don't commit an empty changeset.

**No open PR:** If `gh pr view` fails because there's no PR for the current branch, inform the user. Don't attempt to create a PR — that's a separate decision.

**No review comments:** Perfectly fine. Skip steps 5–7 and go straight to tests.

**Conflicting review feedback:** If two reviewers give contradictory feedback, flag both for the user and ask how to proceed rather than picking a side.

**Tests fail after fixes:** Attempt to fix the failure. If it's not straightforward, report the failure clearly and let the user decide.

**Large number of comments:** If there are more than 10 unresolved comments, present the full list and ask the user if they want to address all of them or prioritize a subset.
