---
name: pr-squash-merge
description: Squash-merge a PR after verifying all changes are committed and pushed, the PR exists, and all review comments are resolved. Then switch to main and pull. Use when the user wants to merge and wrap up a feature branch.
---

# PR Squash Merge

A skill for safely squash-merging a PR once all work is complete. Runs through a series of precondition checks — clean working tree, changes pushed, PR exists, comments resolved — before merging, then brings the local main branch up to date.

## When to Use

- The user says "merge the PR", "squash merge", "we're done with this branch", or "land this"
- The feature branch is ready to merge and the user wants the full merge ritual handled

## Prerequisites

- **`gh` CLI** installed and authenticated
- The current branch is a feature branch (not `main` or `master`)
- The repository uses squash merges (this skill always squash-merges)

## Workflow

### Step 1: Ensure All Changes Are Committed

Check for uncommitted work:

```bash
git status --porcelain
```

If the output is non-empty, there are uncommitted changes. Categorize what's there:

- **Staged but not committed** — ask the user if they want to commit these before merging
- **Unstaged modifications** — ask the user if these should be staged and committed, or discarded
- **Untracked files** — mention them but don't block; untracked files often aren't relevant to the PR

**Do NOT proceed with the merge until the working tree is clean** (no staged or unstaged modifications). Untracked files are acceptable.

```bash
# Verify clean state (ignoring untracked)
test -z "$(git status --porcelain -uno)" && echo "Clean" || echo "Dirty"
```

### Step 2: Ensure All Changes Are Pushed

Compare the local branch to its remote tracking branch:

```bash
BRANCH=$(git branch --show-current)
git fetch origin "$BRANCH" 2>/dev/null

LOCAL=$(git rev-parse HEAD)
REMOTE=$(git rev-parse "origin/$BRANCH" 2>/dev/null)
```

If the remote ref doesn't exist, the branch has never been pushed — push it:

```bash
git push -u origin "$BRANCH"
```

If the remote ref exists but differs from local, check the direction:

```bash
# Commits on local not on remote (need to push)
git log --oneline "origin/$BRANCH..HEAD"

# Commits on remote not on local (need to pull)
git log --oneline "HEAD..origin/$BRANCH"
```

- **Local is ahead** — push: `git push origin "$BRANCH"`
- **Remote is ahead** — pull with rebase: `git pull --rebase origin "$BRANCH"`, then push if needed
- **Diverged** — inform the user; they need to decide how to reconcile before merging

**Do NOT proceed until local and remote are in sync.**

### Step 3: Ensure a PR Exists

Check for an open PR on the current branch:

```bash
gh pr view --json number,title,state,url
```

If this fails or returns a closed/merged PR, there is no open PR for the branch. Inform the user and stop — creating a PR is a separate decision with its own title, description, and reviewer choices.

If a PR exists, capture its details:

```bash
PR_NUMBER=$(gh pr view --json number --jq '.number')
PR_TITLE=$(gh pr view --json title --jq '.title')
PR_URL=$(gh pr view --json url --jq '.url')
```

### Step 4: Ensure All PR Comments Are Resolved

Fetch review comments and check for unresolved threads:

```bash
OWNER=$(gh repo view --json owner --jq '.owner.login')
REPO=$(gh repo view --json name --jq '.name')

# Fetch all review comments
gh api "repos/$OWNER/$REPO/pulls/$PR_NUMBER/comments" --paginate
```

Build a picture of comment threads:

- Top-level comments have no `in_reply_to_id`
- Replies have `in_reply_to_id` pointing to the parent
- A thread is **resolved** if a reply contains "Fixed:" or the thread was resolved in the GitHub UI

Also check for pending reviews that haven't been submitted:

```bash
gh pr view --json reviews --jq '.reviews[] | select(.state == "CHANGES_REQUESTED") | .author.login'
```

If there are unresolved comments or outstanding change requests:

```
⚠️  Cannot merge — there are unresolved review items:

Unresolved comments (2):
- [#38] handler.rs:42 — "This unwrap could panic" (@reviewer)
- [#41] api.rs:118 — "Missing input validation" (@copilot)

Change requests from: @senior-dev

Please resolve these before merging, or confirm you want to merge anyway.
```

**Stop and let the user decide.** If they explicitly say to merge anyway, proceed — but flag it clearly.

### Step 5: Squash Merge the PR

Check that CI/status checks are passing:

```bash
gh pr checks
```

If checks are failing, report which ones and ask the user whether to proceed. Some teams allow merging with advisory (non-required) checks failing.

Perform the squash merge:

```bash
gh pr merge "$PR_NUMBER" --squash --delete-branch
```

This will:
- Squash all commits into a single commit on the target branch
- Use the PR title as the default commit message
- Delete the remote feature branch after merging

If the user wants a custom commit message instead of the PR title:

```bash
gh pr merge "$PR_NUMBER" --squash --delete-branch --subject "<custom title>" --body "<custom body>"
```

If the merge fails due to conflicts, report the error and stop. The user needs to resolve conflicts on the feature branch, push, and try again.

### Step 6: Switch to Main Locally

Determine the default branch name:

```bash
DEFAULT_BRANCH=$(gh repo view --json defaultBranchRef --jq '.defaultBranchRef.name')
```

Switch to it:

```bash
git switch "$DEFAULT_BRANCH"
```

Clean up the local feature branch (the remote was already deleted by `--delete-branch`):

```bash
git branch -d "$BRANCH"
```

If the delete fails because Git thinks the branch isn't fully merged (rare after a squash merge since the histories differ), use:

```bash
git branch -D "$BRANCH"
```

### Step 7: Pull Latest from Remote

Pull the main branch to bring in the squash-merged commit:

```bash
git pull origin "$DEFAULT_BRANCH"
```

Prune any stale remote tracking branches:

```bash
git fetch --prune
```

### Step 8: Summarize

```
## PR Merge Summary

**PR:** #142 — "Add session timeout handling" 
**URL:** https://github.com/org/repo/pull/142
**Merged:** squash merge into main ✅
**Remote branch:** deleted ✅
**Local branch:** deleted ✅
**Local main:** up to date ✅

Squash commit: abc1234 — "Add session timeout handling (#142)"
```

## Edge Cases

**Protected branch rules:** If the target branch has protection rules requiring approvals or passing checks, `gh pr merge` will fail with a clear error. Report the specific requirement that isn't met (e.g., "Requires 1 approving review").

**Merge conflicts:** If the PR has conflicts with the target branch, the merge will be rejected. Inform the user they need to rebase or merge the target branch into their feature branch first.

**Default branch isn't `main`:** The skill uses `gh repo view` to detect the actual default branch name — it works whether it's `main`, `master`, `develop`, or anything else.

**PR was already merged:** If `gh pr view` shows the PR state is `MERGED`, inform the user and skip to Step 6 to switch branches and pull.

**User is on main:** If the current branch is already the default branch, there's nothing to merge. Inform the user and stop.

**Auto-merge enabled:** Some PRs have auto-merge configured to merge when checks pass. If the user wants to wait for auto-merge instead of forcing it now, respect that and just report the current status.