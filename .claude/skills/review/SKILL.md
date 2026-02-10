# PR Review Skill

1. Fetch all open review comments: `gh api repos/{owner}/{repo}/pulls/{pr}/comments`
2. Group comments by severity (CRITICAL, HIGH, MEDIUM, LOW)
3. Fix CRITICAL and HIGH issues first, one commit per logical fix
4. Reply to each resolved comment using `gh api repos/{owner}/{repo}/pulls/{pr}/comments -f body="Fixed: <description>" -F in_reply_to=<comment_id>`
5. Run full test suite after all fixes
6. Push and summarize remaining unresolved items

IMPORTANT: Use `in_reply_to` parameter for reply threading. Do NOT use issues/comments endpoint.
