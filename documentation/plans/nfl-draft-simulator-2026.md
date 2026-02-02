# NFL Draft Simulator 2026 - Implementation Plan

## Overview

Build an NFL Draft simulator with:
- **Backend**: Rust with Axum web framework
- **Database**: PostgreSQL 18
- **Features**: Draft order management, player scouting, AI draft engine, real-time draft room

## Architecture

### Project Structure - Cargo Workspace

```
nfl-draft-2026/
├── Cargo.toml                    # Workspace root
├── migrations/                   # Database migrations (sqlx)
├── crates/
│   ├── api/                      # Axum API server
│   ├── domain/                   # Business logic & models
│   ├── db/                       # Database layer (sqlx)
│   └── websocket/                # WebSocket connections
└── tests/                        # Integration tests
```

**Benefits of workspace approach**:
- Clear separation of concerns (API, domain, database)
- Independent testing of each layer
- Shared dependencies via workspace configuration
- Incremental compilation and testing

### Core Technology Stack

- **Web Framework**: Axum 0.8 (modern, ergonomic, tokio-based)
- **Database**: SQLx 0.8 with PostgreSQL (async, compile-time query verification)
- **Async Runtime**: Tokio 1.x
- **WebSocket**: tokio-tungstenite 0.24
- **Serialization**: Serde 1.0
- **Error Handling**: thiserror 2.0, anyhow 1.0
- **Tracing**: tracing 0.1, tracing-subscriber 0.3

### Database Schema (Core Tables)

#### Teams & Organizations
- `teams` - NFL team information (name, city, conference, division)

#### Players & Scouting
- `players` - Player profiles (name, position, college, physical stats)
- `combine_results` - NFL combine performance data
- `scouting_reports` - Team-specific player evaluations and grades
- `team_needs` - Position priorities for each team

#### Draft Management
- `drafts` - Draft year and status tracking
- `draft_picks` - Draft order (round, pick number, team ownership)
- `pick_trades` - Trade transactions between teams
- `pick_trade_details` - Individual picks involved in trades

#### Real-time Sessions
- `draft_sessions` - Live draft instances with settings
- `draft_events` - Event log for audit trail and replay (JSONB)

### API Design

**REST Endpoints**:
```
Health:
  GET  /health
  GET  /api/v1/version

Teams:
  GET  /api/v1/teams
  GET  /api/v1/teams/:id
  GET  /api/v1/teams/:id/needs
  PUT  /api/v1/teams/:id/needs

Players:
  GET    /api/v1/players
  POST   /api/v1/players
  GET    /api/v1/players/:id
  GET    /api/v1/players/:id/combine
  POST   /api/v1/players/:id/combine
  GET    /api/v1/players/:id/scouting
  POST   /api/v1/players/:id/scouting

Drafts:
  GET    /api/v1/drafts
  POST   /api/v1/drafts
  GET    /api/v1/drafts/:id
  GET    /api/v1/drafts/:id/picks
  POST   /api/v1/drafts/:id/picks

Draft Sessions:
  POST   /api/v1/drafts/:id/sessions
  GET    /api/v1/sessions/:id
  POST   /api/v1/sessions/:id/start
  POST   /api/v1/sessions/:id/pause
  POST   /api/v1/sessions/:id/pick
  POST   /api/v1/sessions/:id/trade
  GET    /api/v1/sessions/:id/events

WebSocket:
  WS     /ws/sessions/:id
```

**WebSocket Protocol**:
- Client → Server: `subscribe`, `make_pick`, `propose_trade`
- Server → Client: `pick_made`, `clock_update`, `trade_executed`, `draft_status`

### Layer Architecture

```
API Layer (Axum)
  ↓ Routes & handlers, request validation
Service Layer (Domain)
  ↓ Business logic, draft engine, validation
Repository Layer (DB)
  ↓ Data access, SQL queries, transactions
PostgreSQL Database
```

**Key Pattern**: Repository trait pattern for testability
- Domain defines repository traits
- DB crate implements traits with SQLx
- Services depend on traits (dependency injection)
- Tests use mock implementations

## Implementation Phases

### Phase 1: Foundation (First Priority)

**Goal**: Project setup and basic CRUD operations

**Tasks**:
1. **Workspace Setup**
   - Create root `Cargo.toml` with workspace configuration
   - Define shared dependencies
   - Create crate structure: `api/`, `domain/`, `db/`, `websocket/`

2. **Database Foundation**
   - Install `sqlx-cli`: `cargo install sqlx-cli --features postgres`
   - Create initial migrations: teams, players
   - Setup connection pooling in `db` crate
   - Implement basic repository traits and implementations

3. **API Scaffolding**
   - Setup Axum server in `api` crate
   - Configure middleware (CORS, logging, error handling)
   - Health check endpoint
   - Application state structure

4. **Basic CRUD Endpoints**
   - Teams: GET /teams, GET /teams/:id
   - Players: GET /players, POST /players, GET /players/:id

**Critical Files**:
- `/Cargo.toml` - Workspace configuration
- `/migrations/20260201000001_create_teams.sql`
- `/migrations/20260201000002_create_players.sql`
- `/crates/domain/src/models/team.rs`
- `/crates/domain/src/models/player.rs`
- `/crates/domain/src/repositories/team.rs` (trait)
- `/crates/db/src/repositories/team_repo.rs` (impl)
- `/crates/api/src/main.rs`
- `/crates/api/src/handlers/teams.rs`

**Validation**: Can start server, access health check, create/read teams and players

### Phase 2: Draft Core

**Goal**: Draft order management and basic draft flow

**Tasks**:
1. Create draft-related migrations (drafts, draft_picks)
2. Implement draft domain models and repositories
3. Build draft creation and initialization logic
4. Add draft endpoints (create, get order, view available players)
5. Implement draft session basics (create, start/pause)

**Critical Files**:
- `/migrations/20260201000003_create_drafts.sql`
- `/crates/domain/src/models/draft.rs`
- `/crates/domain/src/services/draft_engine.rs`
- `/crates/api/src/handlers/drafts.rs`

**Validation**: Can create a draft, initialize pick order, view draft state

### Phase 3: Scouting System

**Goal**: Player evaluation and team needs management

**Tasks**:
1. Create scouting migrations (combine_results, scouting_reports, team_needs)
2. Implement scouting models and repositories
3. Build player ranking logic (BPA, position rankings)
4. Add scouting endpoints

**Critical Files**:
- `/migrations/20260201000004_create_scouting.sql`
- `/crates/domain/src/models/scouting.rs`
- `/crates/domain/src/services/player_ranking.rs`
- `/crates/api/src/handlers/players.rs` (extend)

**Validation**: Can add combine results, create scouting reports, rank players

### Phase 4: Real-time Draft Room

**Goal**: WebSocket integration for live drafts

**Tasks**:
1. Implement WebSocket connection manager in `websocket` crate
2. Create message protocol types
3. Build broadcasting system
4. Add WebSocket endpoint to API
5. Implement draft clock and real-time pick notifications
6. Create draft event recording

**Critical Files**:
- `/crates/websocket/src/manager.rs`
- `/crates/websocket/src/messages.rs`
- `/crates/api/src/handlers/websocket.rs`
- `/migrations/20260201000005_create_sessions.sql`

**Validation**: Multiple clients can connect, receive real-time pick updates, see clock countdown

### Phase 5: AI Draft Engine

**Goal**: Automated team decision-making

**Tasks**:
1. Implement team needs analysis service
2. Build BPA (Best Player Available) logic
3. Create position value calculations
4. Implement auto-pick decision making
5. Add draft strategy configurations

**Critical Files**:
- `/crates/domain/src/services/draft_engine.rs` (extend)
- `/crates/domain/src/services/team_needs.rs`
- `/crates/domain/src/services/player_ranking.rs` (extend)

**Validation**: AI can make realistic draft picks based on team needs and BPA

### Phase 6: Trade Engine

**Goal**: Pick trading with validation

**Tasks**:
1. Create trade migrations (pick_trades, pick_trade_details)
2. Implement trade value calculation
3. Build trade validation logic
4. Add trade execution with transaction handling
5. Create trade endpoints

**Critical Files**:
- `/migrations/20260201000006_create_trades.sql`
- `/crates/domain/src/services/trade_engine.rs`
- `/crates/api/src/handlers/trades.rs`

**Validation**: Can propose trades, validate fairness, execute trades, update pick ownership

### Phase 7: Production Readiness

**Goal**: Polish, performance, observability

**Tasks**:
1. Add structured logging with tracing
2. Implement comprehensive error handling
3. Add database query optimization
4. Create integration test suite
5. Add API documentation (OpenAPI/Swagger)
6. Setup configuration management
7. Add authentication/authorization (if needed)

## Key Architectural Decisions

### 1. Multi-crate Workspace
**Why**: Clear separation of concerns, independent testing, better compile times
**Trade-off**: More complex structure, but worth it for medium/large projects

### 2. SQLx over Diesel
**Why**: Async-first, compile-time query verification, lighter weight
**Trade-off**: Less type safety than Diesel, more manual SQL

### 3. Repository Pattern
**Why**: Enables testing with mocks, decouples domain from database
**Trade-off**: More boilerplate, but essential for testability

### 4. Event Sourcing for Draft Events
**Why**: Complete audit trail, replay capability, flexible schema with JSONB
**Trade-off**: Slightly more storage, but invaluable for debugging

### 5. In-memory WebSocket Manager (DashMap)
**Why**: Simple, performant for single-server setup
**Trade-off**: Doesn't scale horizontally (can add Redis later if needed)

## Testing Strategy

### Unit Tests
- Domain services with mock repositories
- Message serialization/deserialization
- Business logic validation

### Integration Tests
- Full API endpoint testing
- Database transaction testing
- WebSocket message flow

### Test Database
- Use separate `TEST_DATABASE_URL`
- Run migrations before tests
- Clean up data after tests

## Development Workflow

1. **Setup environment**:
   ```bash
   # Install Rust toolchain
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Install sqlx-cli
   cargo install sqlx-cli --no-default-features --features postgres

   # Setup PostgreSQL 18
   # Create database: nfl_draft

   # Setup environment variables
   cp .env.example .env
   # Edit .env with DATABASE_URL
   ```

2. **Run migrations**:
   ```bash
   sqlx migrate run
   ```

3. **Development**:
   ```bash
   cargo build --workspace
   cargo test --workspace
   cargo run -p api
   ```

4. **Incremental approach**:
   - Build domain models first
   - Implement repositories
   - Add API handlers
   - Write tests as you go

## Critical Files Reference

### Phase 1 (Immediate)
- `/Cargo.toml` - Workspace root
- `/crates/api/src/main.rs` - Entry point
- `/crates/domain/src/models/team.rs` - Team model
- `/crates/domain/src/models/player.rs` - Player model
- `/crates/db/src/pool.rs` - Database connection
- `/migrations/20260201000001_create_teams.sql`

### Core Domain Logic
- `/crates/domain/src/services/draft_engine.rs` - Draft simulation
- `/crates/domain/src/services/trade_engine.rs` - Trade logic
- `/crates/domain/src/services/player_ranking.rs` - Rankings

### API Layer
- `/crates/api/src/handlers/drafts.rs` - Draft endpoints
- `/crates/api/src/handlers/websocket.rs` - WebSocket handler

## Verification Plan

**Phase 1 Verification**:
- Start server successfully
- Access health check: `curl http://localhost:8000/health`
- Create team via API
- Query teams and see created team

**Phase 2 Verification**:
- Create draft for 2026
- Initialize draft picks (7 rounds × 32 teams = 224 picks)
- Query draft order and verify correct sequence

**Phase 3 Verification**:
- Add combine results for players
- Create scouting reports with grades
- Query ranked players by position

**Phase 4 Verification**:
- Connect WebSocket client to draft session
- Make a pick and verify client receives `pick_made` message
- Verify clock countdown updates sent to all clients

**Phase 5 Verification**:
- Enable auto-pick for multiple teams
- Start draft session
- Verify AI makes reasonable picks based on team needs

**Phase 6 Verification**:
- Propose pick trade between teams
- Verify trade validation (no duplicate picks)
- Execute trade and verify pick ownership changes

## Notes

- Follow existing Rust conventions (use `cargo fmt`, `cargo clippy`)
- Commit working code incrementally after each sub-phase
- Update tests as features are added
- Keep domain logic separate from API and database concerns
- Use transactions for operations that modify multiple tables
- Log important events for debugging and audit

---

# Frontend Architecture

## Overview

The frontend is built with **SvelteKit + TypeScript + Tailwind CSS**, providing a modern, responsive interface for the NFL Draft simulator.

### Technology Stack

- **Framework**: SvelteKit (latest 2026)
- **Language**: TypeScript
- **Styling**: Tailwind CSS with custom design system
- **State Management**: Svelte 5 runes (modern reactive state)
- **Testing**: Vitest (unit/component), Playwright (E2E)
- **Real-time**: Native WebSocket client with reconnection

## Project Structure

```
frontend/
├── src/
│   ├── lib/
│   │   ├── components/          # Feature-organized components
│   │   │   ├── draft/           # Draft board, pick timer, live feed
│   │   │   ├── player/          # Player cards, search, filters
│   │   │   ├── team/            # Team dashboard, needs config
│   │   │   ├── session/         # Session setup, settings
│   │   │   └── ui/              # Shared UI (Button, Card, Modal)
│   │   ├── stores/              # Svelte 5 runes-based state
│   │   │   ├── draft.svelte.ts
│   │   │   ├── players.svelte.ts
│   │   │   ├── teams.svelte.ts
│   │   │   └── websocket.svelte.ts
│   │   ├── api/                 # API client layer
│   │   │   ├── client.ts        # Base HTTP client
│   │   │   ├── draft.ts
│   │   │   ├── players.ts
│   │   │   ├── teams.ts
│   │   │   └── websocket.ts     # WebSocket client
│   │   ├── types/               # TypeScript definitions
│   │   │   ├── api.ts
│   │   │   ├── draft.ts
│   │   │   ├── player.ts
│   │   │   ├── team.ts
│   │   │   └── websocket.ts
│   │   └── utils/               # Helpers, formatting, validation
│   ├── routes/                  # SvelteKit file-based routing
│   │   ├── +layout.svelte
│   │   ├── +page.svelte         # Home
│   │   ├── draft/
│   │   │   └── [sessionId]/     # Draft room
│   │   ├── players/
│   │   │   └── [playerId]/      # Player detail
│   │   ├── teams/
│   │   │   └── [teamId]/        # Team dashboard
│   │   └── sessions/            # Session management
│   └── app.css                  # Global Tailwind styles
├── tests/
│   ├── unit/                    # Vitest unit tests
│   ├── integration/             # Vitest browser mode
│   └── e2e/                     # Playwright E2E
├── svelte.config.js
├── vite.config.ts
├── tailwind.config.js
└── package.json
```

## Component Architecture

### Design Patterns

- **Smart Components**: Manage state (in route pages)
- **Presentational Components**: Pure UI (in `lib/components`)
- **Domain Organization**: Components grouped by feature (draft, player, team)

### Key Components

**Draft Room**:
- `DraftBoard.svelte` - Main draft board display
- `TeamCard.svelte` - Individual team cards with picks
- `PickTimer.svelte` - Countdown timer
- `DraftOrder.svelte` - Visual pick order
- `LivePickFeed.svelte` - Real-time pick notifications

**Player Interface**:
- `PlayerCard.svelte` - Player card with stats
- `PlayerList.svelte` - Virtualized player list
- `PlayerSearch.svelte` - Search with filters
- `CombineResults.svelte` - Combine data display
- `ScoutingGrade.svelte` - Visual grade representation

**Team Management**:
- `TeamDashboard.svelte` - Team overview
- `TeamRoster.svelte` - Current roster display
- `TeamNeeds.svelte` - Position needs editor
- `StrategyConfig.svelte` - Draft strategy settings

## State Management (Svelte 5 Runes)

Modern approach using runes instead of traditional stores:

```typescript
// lib/stores/draft.svelte.ts
export class DraftState {
  currentPick = $state<number>(1);
  draftOrder = $state<Team[]>([]);
  picks = $state<DraftPick[]>([]);
  currentTimer = $state<number>(0);
  isActive = $state<boolean>(false);

  get currentTeam() {
    return this.draftOrder[this.currentPick - 1];
  }

  makePick(pick: DraftPick) {
    this.picks.push(pick);
    this.currentPick++;
  }
}

export const draftState = new DraftState();
```

**State Stores**:
- `draft.svelte.ts` - Draft session state, picks, timer
- `players.svelte.ts` - Player data, filters, search
- `teams.svelte.ts` - Team information, needs
- `websocket.svelte.ts` - Connection status, messages

## API Client Design

### Base Client

```typescript
// lib/api/client.ts
class ApiClient {
  private baseUrl = '/api/v1';

  async request<T>(endpoint: string, options?: RequestInit): Promise<ApiResponse<T>>
  async get<T>(endpoint: string): Promise<ApiResponse<T>>
  async post<T>(endpoint: string, body: unknown): Promise<ApiResponse<T>>
  async put<T>(endpoint: string, body: unknown): Promise<ApiResponse<T>>
  async delete<T>(endpoint: string): Promise<ApiResponse<T>>
}
```

### Domain APIs

- `draft.ts` - Draft session operations
- `players.ts` - Player CRUD and search
- `teams.ts` - Team management
- `websocket.ts` - Real-time connection

### WebSocket Client

```typescript
// lib/api/websocket.ts
export class WebSocketClient {
  connect(sessionId: string)
  disconnect()
  send<T>(type: string, data: T)
  on<T>(type: string, handler: (data: T) => void)
  off<T>(type: string, handler: (data: T) => void)
  // Auto-reconnection with exponential backoff
}
```

## TypeScript Type System

Types mirror Rust backend structs for consistency:

```typescript
// lib/types/draft.ts
export interface Draft {
  id: string;
  sessionId: string;
  settings: DraftSettings;
  status: DraftStatus;
  currentPick: number;
  draftOrder: string[];
  picks: DraftPick[];
}

export interface DraftPick {
  id: string;
  round: number;
  pickNumber: number;
  teamId: string;
  playerId: string;
  pickTime: number;
  pickedAt: string;
}
```

**Type Modules**:
- `api.ts` - API responses, errors, pagination
- `draft.ts` - Draft domain types
- `player.ts` - Player, combine, scouting types
- `team.ts` - Team, needs, strategy types
- `websocket.ts` - WebSocket message types

## Routing (SvelteKit File-Based)

```
/                          → Home page
/draft                     → Draft lobby
/draft/[sessionId]         → Draft room (with WebSocket)
/players                   → Player list
/players/[playerId]        → Player detail
/teams                     → Team list
/teams/[teamId]            → Team dashboard
/sessions                  → User sessions
/sessions/create           → Create session
/sessions/[sessionId]      → Session detail
```

## Tailwind Design System

### Custom Theme

```javascript
// tailwind.config.js
module.exports = {
  theme: {
    extend: {
      colors: {
        primary: { /* Custom blue palette */ },
        draft: {
          active: '#10b981',
          paused: '#f59e0b',
          completed: '#6b7280',
        },
        position: {
          offense: '#3b82f6',
          defense: '#ef4444',
          special: '#8b5cf6',
        },
      },
      animation: {
        'slide-in': 'slideIn 0.3s ease-out',
        'fade-in': 'fadeIn 0.2s ease-in',
      },
    },
  },
};
```

### Responsive Strategy

- **Mobile** (< 640px): Single column, bottom sheets
- **Tablet** (640px - 1024px): Two columns, side panels
- **Desktop** (> 1024px): Full grid, multi-panel

## WebSocket Integration

### Connection Management

```svelte
<!-- routes/draft/[sessionId]/+layout.svelte -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { wsClient } from '$lib/api/websocket';
  import { draftState } from '$lib/stores/draft.svelte';

  onMount(() => {
    wsClient.connect(sessionId);

    wsClient.on('draft:pick', (data) => {
      draftState.makePick(data.pick);
    });

    wsClient.on('draft:timer', (data) => {
      draftState.updateTimer(data.timeRemaining);
    });
  });

  onDestroy(() => {
    wsClient.disconnect();
  });
</script>
```

### Optimistic Updates

```typescript
async function makePick(playerId: string) {
  // Optimistic update
  draftState.makePick({ playerId, ... });

  // Send to server
  const { error } = await draftApi.makePick(sessionId, { playerId });

  // Rollback on error
  if (error) {
    draftState.rollbackLastPick();
  }
}
```

## Testing Strategy

### Test Types

**Unit Tests** (Vitest):
- Utility functions
- Type guards
- Formatters
- Validation logic

**Component Tests** (Vitest Browser Mode):
- Component rendering
- User interactions
- State changes

**E2E Tests** (Playwright):
- Complete user flows
- Real-time features
- Multi-page scenarios

### Example Tests

```typescript
// tests/unit/utils/format.test.ts
import { formatPickTime } from '$lib/utils/format';

test('formats seconds to MM:SS', () => {
  expect(formatPickTime(90)).toBe('01:30');
});
```

```typescript
// tests/e2e/draft.spec.ts
test('complete draft flow', async ({ page }) => {
  await page.goto('/draft/test-session');
  await page.click('[data-testid="player-card-1"]');
  await page.click('[data-testid="confirm-pick"]');
  await expect(page.locator('.live-pick-feed')).toContainText('John Doe');
});
```

## Frontend Implementation Phases

### Phase F1: Foundation (Week 1)
**After Backend Phase 1 completes**

**Tasks**:
1. Initialize SvelteKit project with TypeScript
2. Configure Tailwind CSS with custom theme
3. Create base API client
4. Define shared TypeScript types
5. Build core UI components (Button, Card, Modal)

**Deliverables**: Working SvelteKit app with API integration

### Phase F2: Player & Team Pages (Week 2)
**Aligns with Backend Phase 2-3**

**Tasks**:
1. Implement player listing and search
2. Build player detail pages
3. Create team dashboard
4. Add player filters and sorting
5. Team needs configuration UI

**Deliverables**: Player and team management interfaces

### Phase F3: Draft Room UI (Week 3)
**Aligns with Backend Phase 2-4**

**Tasks**:
1. Create draft session setup flow
2. Build draft board layout
3. Implement team cards
4. Add pick timer component
5. Create draft order display

**Deliverables**: Complete draft room interface (non-real-time)

### Phase F4: Real-Time Integration (Week 4)
**Aligns with Backend Phase 4**

**Tasks**:
1. Implement WebSocket client
2. Integrate real-time updates
3. Add optimistic UI updates
4. Handle connection errors
5. Implement reconnection logic

**Deliverables**: Live draft room with real-time updates

### Phase F5: Polish & Testing (Week 5)
**Aligns with Backend Phase 7**

**Tasks**:
1. Responsive design refinements
2. Performance optimizations (virtual scrolling)
3. Write unit and component tests
4. Create E2E test suite
5. Accessibility improvements

**Deliverables**: Production-ready frontend

## Key Frontend Decisions

### 1. Svelte 5 Runes over Traditional Stores
**Why**: Modern approach, better TypeScript support, universal reactivity
**Trade-off**: Newer pattern, less community examples

### 2. Domain-Organized Components
**Why**: Better organization, clear boundaries
**Trade-off**: Some shared components across domains

### 3. Optimistic UI Updates
**Why**: Immediate feedback, better UX for real-time actions
**Trade-off**: Requires rollback logic

### 4. Mobile-First Responsive
**Why**: Many users on mobile devices
**Trade-off**: May limit initial desktop features

### 5. Tailwind + Custom Design System
**Why**: Rapid development with brand consistency
**Trade-off**: Large class strings, learning curve

## Critical Frontend Files

### Phase F1 (Immediate)
- `frontend/src/lib/api/client.ts` - Base HTTP client
- `frontend/src/lib/types/draft.ts` - Core type definitions
- `frontend/tailwind.config.js` - Design system
- `frontend/src/routes/+layout.svelte` - Root layout

### Core Features
- `frontend/src/lib/stores/draft.svelte.ts` - Draft state management
- `frontend/src/lib/api/websocket.ts` - WebSocket client
- `frontend/src/routes/draft/[sessionId]/+page.svelte` - Draft room
- `frontend/src/lib/components/draft/DraftBoard.svelte` - Main draft interface

## Frontend Development Workflow

```bash
# Install dependencies
cd frontend
npm install

# Development server
npm run dev

# Type checking
npm run check

# Run tests
npm run test           # Unit tests
npm run test:e2e       # E2E tests

# Build for production
npm run build
npm run preview        # Preview build
```

## Integration with Backend

### API Proxy (Development)

```javascript
// svelte.config.js
export default {
  kit: {
    adapter: adapter(),
    alias: {
      $lib: 'src/lib'
    },
    // Proxy API requests to Rust backend
    server: {
      proxy: {
        '/api': 'http://localhost:8000',
        '/ws': {
          target: 'ws://localhost:8000',
          ws: true
        }
      }
    }
  }
};
```

### Production Deployment

- Frontend: Static SvelteKit build
- Backend: Axum server serves both API and static files
- WebSocket: Same origin for simplified CORS

## Verification Plan (Frontend)

**Phase F1**:
- Run dev server successfully
- Navigate to home page
- Verify Tailwind styles working

**Phase F2**:
- View player list with search
- Filter players by position
- Navigate to player detail
- View team dashboard

**Phase F3**:
- Create new draft session
- View draft board
- See team cards in correct order
- Timer displays correctly

**Phase F4**:
- Connect to WebSocket
- Receive real-time pick updates
- Timer syncs across multiple clients
- Reconnects after disconnect

**Phase F5**:
- Test responsive layouts (mobile, tablet, desktop)
- All E2E tests pass
- No accessibility violations
- Performance metrics acceptable
