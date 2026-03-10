---
name: self-review
description: Self-review all changes on the current branch vs main. Prompt for model choice, then deeply review implementation quality, correctness, and test coverage. Fix issues and add missing tests with individual commits. Summarize findings at the end.
---

# Self-Review

A skill for performing a thorough self-review of all changes on the current feature branch compared to main. Focuses on implementation correctness, code quality, and test coverage gaps.

## When to Use

- The user says "self-review", "review my changes", "review this branch", or "check my work"
- Before opening or merging a PR, to catch issues early
- After a batch of changes when the user wants a quality check

## Prerequisites

- Must be on a feature branch (not main)
- Branch must have commits ahead of main
- Working directory should be clean (no uncommitted changes that would interfere with review)

## Workflow

### Step 0: Prompt for Model

Before starting the review, ask the user which model they'd like to use for the review. Present options using AskUserQuestion:

- **Opus** (Recommended) — Deepest analysis, best for thorough reviews
- **Sonnet** — Good balance of speed and depth

This allows the user to choose the level of thoroughness vs speed. Do NOT proceed until the user has selected a model.

### Step 1: Gather the Full Diff

Collect all changes on this branch compared to main:

```bash
git diff main...HEAD
git diff main...HEAD --stat
git log main..HEAD --oneline
```

This gives you the complete picture of what changed — every file, every line.

### Step 2: Identify Changed Files by Category

Organize the changed files into categories to structure the review:

- **Backend Rust** — `back-end/crates/*/src/**/*.rs`
- **Backend Data** — `back-end/data/**/*.json`
- **Migrations** — `back-end/migrations/**/*.sql`
- **Frontend Svelte/TS** — `front-end/src/**/*.{svelte,ts}`
- **Config/Infra** — `docker-compose.yml`, `Cargo.toml`, `package.json`, scripts, etc.
- **Tests** — any file matching `*test*`, `*spec*`, or in a `tests/` directory

### Step 3: Deep Implementation Review

For each changed file, read the full file (not just the diff) to understand context. Think hard about:

#### Correctness
- Does the logic do what it's supposed to do?
- Are there off-by-one errors, missing null checks, or unhandled edge cases?
- Do SQL queries match the schema? Are there missing indexes for new query patterns?
- Are error messages accurate and helpful for debugging?

#### Safety & Security
- Any unwrap() calls that could panic on user input?
- SQL injection risks (should be using parameterized queries)?
- Missing input validation at API boundaries?
- Sensitive data exposure in error messages or logs?

#### Consistency
- Does new code follow existing patterns in the codebase?
- Are naming conventions consistent (snake_case in Rust, camelCase in TS)?
- Do new API endpoints follow the existing URL structure and response format?
- Are domain models validated the same way as existing ones?

#### Architecture
- Does the change respect layer boundaries (API -> Domain -> DB)?
- Are repository traits defined in domain, implementations in db?
- Is business logic in services, not in handlers?
- Are there circular dependencies or inappropriate coupling?

#### Frontend-Specific
- Are Svelte 5 runes used correctly ($state, $derived, $effect)?
- Is TypeScript strict — no `any` types, proper null handling?
- Are Zod schemas validating API responses?
- Accessibility: proper ARIA attributes, keyboard navigation, semantic HTML?

#### Data & Seed Files
- Does JSON data match the expected schema?
- Are counts/totals in metadata accurate?
- Any duplicates or inconsistencies in seed data?

### Step 4: Identify Test Coverage Gaps

This is critical. For every changed file, determine if test coverage exists:

1. **List all changed source files** (exclude test files themselves)
2. **For each source file**, search for corresponding test files:
   - Rust: look for `#[cfg(test)]` modules in the same file, or `tests/` directory
   - Frontend: look for `.test.ts` or `.spec.ts` files alongside the source
3. **For each function/method added or modified**, check if tests exercise it
4. **Identify gaps** — new functions without tests, new branches without coverage, new error paths untested

Present gaps as a numbered list:

```
Test Coverage Gaps Found:
1. [back-end/crates/seed-data/src/foo_loader.rs] — new validate() function has no unit tests
2. [front-end/src/lib/api/foo.ts] — API client module has no test file
3. [back-end/crates/domain/src/models/foo.rs] — edge case: empty string input not tested
```

If no gaps are found, state that explicitly and skip to Step 6.

### Step 5: Fix Issues and Add Tests

For each issue found (implementation bugs, missing tests, quality problems):

1. **Read the relevant code** — understand the full context
2. **Determine the fix** — be precise about what needs to change
3. **Apply the change** — edit the file
4. **Run relevant tests** — verify the fix doesn't break anything
5. **Commit with a descriptive message** — one commit per fix

For implementation fixes:
```bash
git add <file>
git commit -m "fix(<scope>): <what was fixed>

Found during self-review: <brief explanation>"
```

For new tests:
```bash
git add <test-file>
git commit -m "test(<scope>): <what is now tested>

Covers gap found during self-review: <brief explanation>"
```

**Important:**
- One commit per fix or test addition — do not batch
- Run tests after each change to ensure nothing breaks
- For Rust backend tests: `cargo test -p <crate>` for the affected crate
- For frontend tests: `npm run test -- <test-file>` for the affected test

### Step 6: Final Verification

Run the full test suites to confirm everything passes:

**Backend:**
```bash
cd back-end && cargo test -p domain -p seed-data
```

Note: `cargo test --workspace` requires the test database. If `nfl_draft_test` is not available, run only the crates that don't need it (domain, seed-data). Note which crates were skipped.

**Frontend:**
```bash
cd front-end && npm run check && npm run test
```

### Step 7: Summary

Provide a structured summary of everything found and fixed:

```
## Self-Review Summary

**Branch:** feature/foo-bar (N commits ahead of main)
**Files reviewed:** N files across backend/frontend/config

### Implementation Issues Found
- [severity] [file:line] — Description of the issue
  - Status: Fixed in commit abc1234 / Flagged for user

### Test Coverage
- Tests added: N new test(s)
- Gaps remaining: N (if any, list them with justification for why they were left)

### Quality Observations
- [Optional] Notes on code quality, patterns, or suggestions that don't warrant immediate fixes

### Test Results
- Backend (cargo test): pass/fail summary
- Frontend (npm run check): pass/fail
- Frontend (npm run test): pass/fail summary

### Commits Made
- fix(scope): description
- test(scope): description
```

Categorize issues by severity:
- **Critical** — bugs, security issues, data corruption risks (must fix)
- **High** — missing error handling, incorrect logic that could fail in production (should fix)
- **Medium** — inconsistencies, missing validation, style issues (fix if straightforward)
- **Low** — minor suggestions, optional improvements (flag but don't fix)

## Edge Cases

**No changes on branch:** If `git diff main...HEAD` is empty, inform the user there's nothing to review.

**Very large diff (50+ files):** Focus review on non-generated files. Skip reviewing `.sqlx/` cache files, lock files, and auto-generated code. Prioritize business logic and API handlers.

**Tests require database:** If backend integration tests fail due to missing test database, note this in the summary but don't treat it as a code issue. Focus on unit tests that can run without infrastructure.

**Ambiguous issues:** If you're unsure whether something is a bug or intentional, flag it for the user rather than "fixing" it. Don't make assumptions about intent.

**Pre-existing issues:** Only flag issues in code that was changed on this branch. Don't review or fix pre-existing problems in unchanged files — that's out of scope.
