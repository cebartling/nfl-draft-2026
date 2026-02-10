# PR Cycle Skill

1. Commit all staged changes with a conventional commit message
2. Push to the current branch
3. Update the PR description using `gh pr edit` to reflect current changes
4. Run `gh pr review` to check for Copilot review comments
5. List all unresolved review comments
6. Fix each issue, making individual commits per fix
7. Push all fixes
8. Re-run tests (backend: `cargo test --workspace`, frontend: from package.json)
9. Summarize what was fixed and current test status
