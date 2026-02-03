# ADR 0004: SvelteKit Framework with Svelte 5 Runes

## Status

Accepted

## Context

We needed to choose a frontend framework for building a real-time, interactive NFL draft simulator with:

- Real-time updates via WebSocket
- Complex state management (draft state, player data, team needs)
- Type safety with TypeScript
- Good developer experience
- Small bundle size for fast loading
- Server-side rendering capability

Options considered:

1. **React + Next.js**: Industry standard, large ecosystem, mature
2. **Vue + Nuxt**: Progressive framework, good DX, simpler than React
3. **SvelteKit + Svelte 5**: Compiler-based, minimal runtime, modern reactivity
4. **Angular**: Full framework, TypeScript-first, enterprise-ready
5. **Solid.js + SolidStart**: Fine-grained reactivity, similar to React

## Decision

We will use **SvelteKit with Svelte 5 Runes** for the frontend application.

Key technologies:

- **Framework**: SvelteKit 2.x
- **Component Library**: Svelte 5 with Runes
- **Type System**: TypeScript 5.x
- **Styling**: Tailwind CSS 4.x
- **Build Tool**: Vite
- **Testing**: Vitest (unit/integration) + Playwright (E2E)

## Consequences

### Positive

#### Svelte 5 Runes

- **Modern Reactivity**: Runes (`$state`, `$derived`, `$effect`, `$props`) provide explicit, fine-grained reactivity
- **No Virtual DOM**: Svelte compiles to minimal JavaScript, updating the DOM directly
- **Smaller Bundles**: Typical Svelte apps are 30-40% smaller than equivalent React apps
- **Better Performance**: Direct DOM manipulation is faster than virtual DOM diffing
- **Simpler Mental Model**: Less boilerplate, no hooks confusion, clearer data flow

Example of Svelte 5 runes:

```svelte
<script lang="ts">
  let count = $state(0);
  let doubled = $derived(count * 2);

  $effect(() => {
    console.log('Count changed:', count);
  });
</script>
```

vs React:

```tsx
const [count, setCount] = useState(0);
const doubled = useMemo(() => count * 2, [count]);

useEffect(() => {
  console.log("Count changed:", count);
}, [count]);
```

#### SvelteKit Framework

- **File-based Routing**: Routes defined by directory structure in `src/routes/`
- **Server-Side Rendering**: SSR by default for better SEO and initial load
- **API Routes**: Backend logic alongside frontend (though we use separate Rust backend)
- **Built-in TypeScript**: First-class TypeScript support
- **Vite Integration**: Fast HMR (Hot Module Replacement) during development

#### Developer Experience

- **Less Boilerplate**: Svelte components are typically 30-40% fewer lines than React
- **Better Error Messages**: Compiler provides clear, actionable errors
- **Scoped Styles**: CSS is scoped to components by default
- **Template Syntax**: Familiar HTML-like syntax with minimal JavaScript
- **Great Tooling**: Svelte extension for VS Code, svelte-check for type checking

### Negative

- **Smaller Ecosystem**: Fewer third-party libraries compared to React (though growing rapidly)
- **Smaller Community**: Less Stack Overflow content, fewer tutorials
- **Hiring**: Fewer developers know Svelte compared to React
- **Runes Are New**: Svelte 5 runes are relatively new (2024), less proven in production
- **Breaking Changes**: Svelte 5 is a significant change from Svelte 4 (though migration is straightforward)
- **Learning Curve for Runes**: Developers familiar with Svelte 4 need to learn runes

### Neutral

- **Different Paradigm**: Compiler-based approach is different from React/Vue's runtime approach
- **TypeScript Integration**: Requires `svelte-check` in addition to `tsc`
- **Component Structure**: Single-file components (similar to Vue) vs React's JSX

## Implementation Details

### State Management

Svelte 5 runes replace traditional Svelte stores for component-level state:

```typescript
// lib/stores/draft.svelte.ts
export class DraftState {
  currentPick = $state<number>(1);
  picks = $state<DraftPick[]>([]);

  get currentTeam() {
    return this.draftOrder[this.currentPick - 1];
  }

  makePick(pick: DraftPick) {
    this.picks.push(pick);
    this.currentPick++;
  }
}

// Export singleton instance
export const draftState = new DraftState();
```

Usage in components:

```svelte
<script lang="ts">
  import { draftState } from '$stores';

  // Reactive - updates automatically when currentPick changes
  let teamName = $derived(draftState.currentTeam?.name);
</script>
```

### API Integration

Domain-specific API modules match backend structure:

```typescript
// lib/api/drafts.ts
export const draftsApi = {
  async list(): Promise<Draft[]> {
    return apiClient.get("/drafts", DraftSchema.array());
  },

  async getById(id: string): Promise<Draft> {
    return apiClient.get(`/drafts/${id}`, DraftSchema);
  },
};
```

### WebSocket Integration

Type-safe WebSocket client with auto-reconnection:

```typescript
// lib/api/websocket.ts
export class WebSocketClient {
  private ws: WebSocket | null = null;
  private reconnectAttempts = $state(0);

  connect() {
    this.ws = new WebSocket("ws://localhost:8000/ws");
    this.setupHandlers();
  }

  private async reconnect() {
    const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);
    await new Promise((resolve) => setTimeout(resolve, delay));
    this.connect();
  }
}
```

### Testing Strategy

**Unit/Integration Tests (Vitest)**:

```typescript
import { describe, it, expect } from "vitest";
import { DraftState } from "./draft.svelte";

describe("DraftState", () => {
  it("should increment pick on makePick", () => {
    const state = new DraftState();
    state.makePick({
      /* ... */
    });
    expect(state.currentPick).toBe(2);
  });
});
```

**E2E Tests (Playwright)**:

```typescript
import { test, expect } from "@playwright/test";

test("should display draft picks", async ({ page }) => {
  await page.goto("/drafts/123");
  await expect(page.locator(".pick-card")).toHaveCount(5);
});
```

## Alternatives Considered

### React + Next.js

**Pros**: Largest ecosystem, most developers, proven at scale, extensive tooling
**Cons**: Larger bundles, more boilerplate, hooks complexity, virtual DOM overhead
**Rejected**: Bundle size and boilerplate were concerns for developer productivity

### Vue + Nuxt

**Pros**: Good balance between React and Svelte, familiar template syntax, mature
**Cons**: Larger than Svelte, composition API adds complexity similar to React hooks
**Rejected**: Svelte's compiler approach provides better performance and DX

### Angular

**Pros**: Full framework, everything included, TypeScript-first, enterprise features
**Cons**: Very heavy, steep learning curve, opinionated, RxJS complexity
**Rejected**: Too heavy for this project's needs, overkill for the problem domain

### Solid.js + SolidStart

**Pros**: Fine-grained reactivity like Svelte, React-like syntax, no virtual DOM
**Cons**: Much smaller community than Svelte, less mature tooling, newer framework
**Rejected**: Svelte has better tooling and larger community while providing similar benefits

## Migration Path

If we need to migrate away from Svelte in the future:

1. **Component Isolation**: Keep components small and focused for easier porting
2. **API Layer Separation**: API logic is in separate TypeScript modules, not Svelte-specific
3. **Type Safety**: TypeScript types can be reused in other frameworks
4. **WebSocket Client**: Independent of Svelte, can be used with any framework
5. **Tailwind CSS**: Styling is framework-agnostic

## References

- [Svelte 5 Documentation](https://svelte.dev/docs/svelte/overview)
- [SvelteKit Documentation](https://kit.svelte.dev/)
- [Svelte 5 Runes](https://svelte.dev/docs/svelte/what-are-runes)
- [Why Svelte 5 Runes](https://svelte.dev/blog/runes)
- [Svelte vs React Bundle Size](https://blog.logrocket.com/should-you-use-svelte-production/)
