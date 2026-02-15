---
name: feature-demo
description: Demo new web application features using Playwright MCP. Use when a new feature has been implemented and needs a visual walkthrough — launching the app in a browser, navigating to the feature, interacting with UI elements, and capturing screenshots to verify behavior.
---

# Feature Demo with Playwright MCP

A skill for demoing new or changed web application features using the Playwright MCP server. After implementing a feature, use this skill to launch the application in a real browser, walk through the feature step by step, take screenshots at key moments, and verify everything looks and behaves as expected.

## When to Use

- A feature has just been implemented or modified and needs a visual walkthrough
- You want to verify UI behavior beyond what unit/integration tests cover
- You need screenshots to confirm layout, styling, or content rendering
- The user says something like "demo this", "show me how it looks", "walk through the feature", or "take screenshots"

## Prerequisites

- **Playwright MCP server** must be configured and available. If it's not connected, inform the user and suggest they add it to their Claude Code MCP settings.
- The web application must be runnable locally (e.g. via `npm run dev`, `yarn dev`, or similar).

## Workflow

### 1. Understand the Feature

Before launching a browser, understand what you're demoing:

- Read the relevant source files, PR description, or requirements to know what changed
- Identify the URL path(s) and UI elements involved
- Plan the sequence of interactions (clicks, form inputs, navigation) needed to showcase the feature
- Note any specific states or data that need to be set up first

### 2. Start the Application

Ensure the API server and the frontend server are both running. If it's not already up:

```bash
# Adapt to the project's actual start command
./scripts/run-app.sh
```

- This Shell script will start the API server and the front end server in Docker Compose. It will rebuild the OCI-compliant images if it needs to.
- Wait for the server to be ready before proceeding. Check the output for the local URL (web application will listen on `http://localhost:3000`.

### 3. Launch the Browser

Use Playwright MCP to open a browser and navigate to the application:

```
playwright: browser_navigate to <app-url>
```

Take an initial screenshot to confirm the app loaded correctly:

```
playwright: browser_screenshot
```

### 4. Walk Through the Feature

Execute the demo step by step. For each meaningful interaction:

1. **Describe** what you're about to do and why
2. **Act** — click, type, select, scroll, or navigate using Playwright MCP tools:
   - `browser_click` — click buttons, links, menu items
   - `browser_type` — fill in form fields, search bars
   - `browser_select_option` — choose from dropdowns
   - `browser_screenshot` — capture the current state
   - `browser_navigate` — go to a specific URL
   - `browser_scroll` — scroll to reveal content
   - `browser_hover` — trigger hover states, tooltips, menus
   - `browser_wait` — wait for animations, loading states, or async content
3. **Screenshot** — capture the result after each significant state change
4. **Verify** — confirm the UI matches expectations (correct text, visible elements, proper layout)

### 5. Handle Common Scenarios

**Loading states**: Use `browser_wait` or `browser_snapshot` to check the DOM before interacting with elements that may not be immediately available.

**Authentication**: If the app requires login, walk through the auth flow first or note any seed/test credentials the project provides.

**Responsive behavior**: If relevant, resize the viewport with `browser_resize` to demo mobile or tablet layouts.

**Error states**: If the feature includes error handling, demo both the happy path and at least one error/edge case.

**Dark mode / themes**: If the feature is affected by theming, demo it in both modes if applicable.

### 6. Summarize Results

After the walkthrough, provide a clear summary:

- **What was demoed**: The feature and the specific interactions performed
- **What worked**: Confirmed behaviors and UI elements
- **Issues found**: Any visual bugs, broken interactions, or unexpected behavior
- **Screenshots**: Reference the screenshots taken at each step

## Best Practices

- **Take screenshots generously** — capture before and after each interaction so the user has a full visual record
- **Use descriptive step narration** — explain what each action is testing so the demo doubles as documentation
- **Don't skip initial state** — always screenshot the starting point before any interactions
- **Check for regressions** — if you know what the feature looked like before, note any unintended changes in surrounding UI
- **Keep the dev server running** — don't stop the server until the demo is fully complete
- **Clean up** — close the browser when the demo is finished using `browser_close`

## Example Interaction

User: "Demo the new dark mode toggle I just added to the settings page"

Steps you would take:

1. Read the relevant settings page component to understand the implementation
2. Start the dev server if not running
3. Open browser → navigate to the settings page → screenshot
4. Locate the dark mode toggle → screenshot showing the toggle in its default state
5. Click the toggle → screenshot showing the UI in dark mode
6. Navigate to other pages to verify dark mode persists → screenshot each
7. Toggle back to light mode → screenshot to confirm it reverts
8. Summarize: what worked, any styling issues spotted, all screenshots

## Troubleshooting

| Problem                      | Solution                                                                          |
| ---------------------------- | --------------------------------------------------------------------------------- |
| Playwright MCP not available | Check Claude Code MCP config — ensure the Playwright server is listed and enabled |
| App won't start              | Check for port conflicts, missing dependencies (`npm install`), or build errors   |
| Element not found            | Use `browser_snapshot` to inspect the current DOM and find the correct selector   |
| Page loads blank             | Check the browser console for errors using `browser_console_messages`             |
| Timeout on interaction       | Increase wait time or check if the element requires scrolling into view first     |
