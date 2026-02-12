# ADR 0011: Screenplay Pattern for Acceptance Test Organization

## Status

Accepted

## Context

The full-stack acceptance tests (ADR-0010) interact with the system through three distinct channels:

1. **Browser** — Playwright Page for UI interactions
2. **REST API** — `fetch` for fast test setup and direct API validation
3. **Database** — `pg.Pool` for SQL-level data integrity assertions

The standard **Page Object** pattern, widely used in browser testing, models pages as objects with methods for interactions. However, Page Objects are designed for a single interaction channel (the browser) and become awkward when a test also needs to call REST APIs and query databases. This leads to "fat" page objects that mix UI interactions with API calls and SQL queries, violating the single-responsibility principle.

We needed a test organization pattern that:

- Cleanly separates the three interaction channels
- Produces readable tests that express business intent
- Scales to new interaction types without restructuring
- Encourages reusable, composable test building blocks

## Decision

We will organize acceptance tests using the **Screenplay pattern**, which separates concerns into four concepts:

### Actors
Who is performing the test. An Actor has a name and a set of Abilities.

```typescript
const actor = new Actor('DraftManager')
  .whoCan(BrowseTheWeb.using(page), CallApi.at(apiUrl), QueryDatabase.using(pool));
```

### Abilities
How actors interact with the system. Each Ability wraps one interaction channel:

- **BrowseTheWeb** — wraps Playwright `Page`
- **CallApi** — wraps `fetch` for REST API calls
- **QueryDatabase** — wraps `pg.Pool` for SQL queries

### Tasks
What business actions actors perform. Tasks are high-level, composable steps:

- `CreateDraft.named('My Draft').withRounds(3)` — fills form, submits, waits for redirect
- `StartSession.forDraft(draftId)` — creates and starts a session via API
- `Navigate.toTeams()` — browser navigation

### Questions
What actors observe about system state. Questions query one interaction channel:

- `DraftStatus.inDatabaseFor(draftId)` — SQL query via QueryDatabase
- `PageHeading.text()` — DOM query via BrowseTheWeb
- `TeamCount.inDatabase()` — aggregate query via QueryDatabase

### Test readability

Tests read like user stories:

```typescript
await actor.attemptsTo(
  Navigate.to('/drafts/new'),
  CreateDraft.named('My 2026 Draft').withRounds(1)
);
const status = await actor.asks(DraftStatus.inDatabaseFor(draftId));
expect(status).toBe('NotStarted');
```

## Consequences

### Positive

- **Clean channel separation** — Each Ability encapsulates one interaction type; no mixing of concerns
- **Readable tests** — Tests express business intent, not implementation details
- **Composable** — Tasks and Questions are small, reusable building blocks
- **Extensible** — Adding a new channel (e.g., WebSocket) only requires a new Ability class
- **Testable in isolation** — Tasks and Questions can be unit tested with mock Actors

### Negative

- **Learning curve** — Developers unfamiliar with Screenplay need to learn the pattern vocabulary (Actor, Ability, Task, Question)
- **More files** — The pattern creates more small files than a flat test helper approach
- **Indirection** — Simple tests require more ceremony than raw Playwright API calls

### Neutral

- **No external framework dependency** — We implement the pattern ourselves (~50 lines of core code) rather than depending on Serenity/JS or similar frameworks
- **TypeScript-native** — Uses TypeScript interfaces and generics for type safety

## Alternatives Considered

### Page Objects

Model each page as a class with methods for interactions.

**Rejected because**: Page Objects only model browser interactions. Adding API calls and database queries to page objects creates mixed-responsibility classes. A `DraftPage` that also has `getStatusFromDatabase()` and `createViaApi()` methods violates single responsibility and becomes hard to maintain.

### Raw Playwright helpers

Create a flat module of helper functions (e.g., `createDraftViaUi()`, `queryDraftStatus()`).

**Rejected because**: No structure or conventions for organizing helpers; as the test suite grows, helpers become a disorganized grab-bag. The Screenplay pattern provides clear organizational categories.

### Hybrid Page Objects + helpers

Use Page Objects for browser interactions and separate helper modules for API/DB.

**Rejected because**: Adds complexity without the composability benefits of Screenplay. Tests would mix two different patterns, making it unclear where new test logic belongs.

## References

- [Screenplay Pattern - Serenity/JS Handbook](https://serenity-js.org/handbook/design/screenplay-pattern/)
- [The Screenplay Pattern - Cucumber Blog](https://cucumber.io/blog/bdd/understanding-screenplay-(part-1)/)
- [Page Objects vs. Screenplay Pattern](https://janmolak.com/page-objects-revisited-beyond-page-objects-fba89c78b6e9)
