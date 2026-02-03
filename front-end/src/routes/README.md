# Routes Documentation

This directory contains all the routes for the NFL Draft Simulator 2026 application.

## Route Structure

```
src/routes/
├── +layout.svelte          # Root layout with navigation and toast
├── +page.svelte            # Home page with welcome and recent drafts
│
├── drafts/
│   ├── +page.svelte        # List all drafts with filtering
│   └── [id]/
│       └── +page.svelte    # Draft details and board
│
├── sessions/
│   └── [id]/
│       ├── +layout.svelte  # Session lifecycle (WebSocket connect/disconnect)
│       └── +page.svelte    # Draft room with real-time updates
│
├── players/
│   ├── +page.svelte        # Player list with filtering and search
│   └── [id]/
│       └── +page.svelte    # Player details
│
└── teams/
    ├── +page.svelte        # Team list grouped by conference/division
    └── [id]/
        └── +page.svelte    # Team details with needs and picks
```

## Routes

### Root Layout (`+layout.svelte`)

**Purpose**: Provides consistent layout across all pages

**Features**:

- Sticky navigation bar with links to Home, Drafts, Players, Teams
- Responsive mobile menu
- Global toast notifications
- Max-width content container (max-w-7xl)

**Components Used**:

- `Toast` - Global notification system

### Home Page (`+page.svelte`)

**Purpose**: Landing page and dashboard

**Features**:

- Hero section with call-to-action
- Recent drafts (up to 5 most recent)
- Quick statistics (total, active, completed drafts)
- Features overview cards
- Create new draft button

**State Management**:

- Fetches drafts using `draftsApi.list()`
- Sorts by created date (most recent first)

**Components Used**:

- `Card` - Content containers
- `Badge` - Status indicators
- `LoadingSpinner` - Loading states

### Drafts List (`/drafts`)

**Purpose**: View and manage all drafts

**Features**:

- Filter by status (all, pending, active, completed)
- Create new draft button
- Draft cards showing:
  - Year, status, rounds, picks per round
  - Created/updated dates
  - Action buttons (Start Draft / Join Session / View Results)

**State Management**:

- Fetches drafts using `draftsApi.list()`
- Filters locally by status
- Sorts by year (most recent first)

**Components Used**:

- `Card` - Draft cards
- `Badge` - Status badges
- `LoadingSpinner` - Loading states

### Draft Details (`/drafts/[id]`)

**Purpose**: View draft details and results

**Features**:

- Draft header with status and details
- Draft progress bar
- Draft board showing all picks
- Draft statistics (rounds completed, picks made, remaining)
- Start/Join session buttons

**State Management**:

- Fetches draft using `draftsApi.get(draftId)`
- Fetches picks using `draftsApi.getPicks(draftId)`

**Components Used**:

- `DraftBoard` - Display all picks
- `Card` - Content sections
- `Badge` - Status indicators
- `LoadingSpinner` - Loading states

### Session Layout (`/sessions/[id]/+layout.svelte`)

**Purpose**: Manage session lifecycle and WebSocket connection

**Features**:

- Loads draft session on mount
- Connects WebSocket for real-time updates
- Disconnects WebSocket on unmount

**State Management**:

- `draftState.loadSession(sessionId)` - Load session
- `wsState.connect(sessionId)` - Connect WebSocket
- `wsState.disconnect()` - Cleanup on unmount

### Draft Room (`/sessions/[id]`)

**Purpose**: Real-time draft room with live updates

**Features**:

- 3-column layout (desktop) / stacked (mobile):
  - Left: Draft clock, session controls, current pick info, selected player
  - Center: Draft board with all picks
  - Right: Available players list
- Connection status indicator
- Player selection and pick confirmation
- Real-time updates via WebSocket

**State Management**:

- `draftState` - Current session, picks, current team/pick
- `playersState` - All players
- `wsState` - WebSocket connection and messages
- Filters available players by excluding picked players

**Components Used**:

- `DraftClock` - Pick timer
- `SessionControls` - Start/pause/stop controls
- `DraftBoard` - Pick history
- `PlayerList` - Available players
- `Badge` - Connection status
- `LoadingSpinner` - Loading states

**API Calls**:

- `playersState.loadAll()` - Load all players
- `draftsApi.makePick()` - Submit pick

### Players List (`/players`)

**Purpose**: Browse and search players

**Features**:

- Search by name or college
- Filter by position group (offense, defense, special teams)
- Filter by specific position
- Active filters display with clear buttons
- Player count indicator

**State Management**:

- `playersState.loadAll()` - Load all players
- Local filtering and search

**Components Used**:

- `PlayerList` - Display players
- `LoadingSpinner` - Loading states

**Navigation**:

- Click player to view details at `/players/[id]`

### Player Details (`/players/[id]`)

**Purpose**: View detailed player information

**Features**:

- Back button to players list
- Player details with stats, combine results, scouting reports
- Error state for not found

**State Management**:

- `playersState.loadPlayer(playerId)` - Load player

**Components Used**:

- `PlayerDetails` - Display all player information
- `LoadingSpinner` - Loading states

### Teams List (`/teams`)

**Purpose**: View all NFL teams

**Features**:

- Group by conference or division (toggle)
- Organized team cards
- Team count indicator

**State Management**:

- Fetches teams using `teamsApi.list()`
- Groups locally by conference or division

**Components Used**:

- `TeamList` - Display teams
- `LoadingSpinner` - Loading states

**Navigation**:

- Click team to view details at `/teams/[id]`

### Team Details (`/teams/[id]`)

**Purpose**: View team information and needs

**Features**:

- Back button to teams list
- Team information card
- Team needs analysis
- Draft picks (when available)
- Team statistics (placeholder)
- Error state for not found

**State Management**:

- Fetches team using `teamsApi.get(teamId)`
- Loads team picks (when available)

**Components Used**:

- `TeamCard` - Team information
- `TeamNeeds` - Position needs
- `Card` - Content sections
- `Badge` - Status indicators
- `LoadingSpinner` - Loading states

## Design Patterns

### State Management

All routes use Svelte 5 runes (`$state`, `$derived`, `$effect`):

```typescript
let data = $state<Type[]>([]);
let loading = $state(true);
let error = $state<string | null>(null);

let filtered = $derived(() => {
	// Computed value based on state
});

$effect(() => {
	// Side effects when dependencies change
});
```

### API Integration

Routes call API functions from `$api`:

```typescript
import { draftsApi } from '$api/draft';
import { playersApi } from '$api/player';
import { teamsApi } from '$api/team';
```

### Component Imports

Components are imported from `$components`:

```typescript
import Card from '$components/ui/Card.svelte';
import Badge from '$components/ui/Badge.svelte';
import DraftBoard from '$components/draft/DraftBoard.svelte';
```

### Navigation

Routes use `goto()` for programmatic navigation and `<a>` tags for declarative navigation:

```typescript
import { goto } from '$app/navigation';
goto('/drafts/123');
```

### Route Parameters

Dynamic routes use `$page.params`:

```typescript
import { page } from '$app/stores';
let id = $derived($page.params.id);
```

### Lifecycle

Routes use `onMount` and `onDestroy` for lifecycle management:

```typescript
import { onMount, onDestroy } from 'svelte';

onMount(async () => {
	// Load data
});

onDestroy(() => {
	// Cleanup
});
```

## Responsive Design

All routes use Tailwind CSS for responsive design:

- **Mobile-first**: Base styles apply to mobile
- **Breakpoints**:
  - `md:` - 768px and up (tablets)
  - `lg:` - 1024px and up (desktops)
- **Grid layouts**: Responsive column counts
  - Mobile: `grid-cols-1`
  - Tablet: `md:grid-cols-2`
  - Desktop: `lg:grid-cols-3`

## Error Handling

All routes implement consistent error handling:

1. **Loading state**: Show `LoadingSpinner` while fetching
2. **Error state**: Display error message with retry option
3. **Empty state**: Show helpful message when no data
4. **Not found**: Show 404-style error with navigation back

## Future Enhancements

Potential routes to add:

- `/drafts/new` - Create new draft form
- `/sessions/[id]/trades` - Trade management interface
- `/players/compare` - Compare multiple players
- `/teams/[id]/history` - Team draft history
- `/admin` - Admin dashboard
- `/settings` - User preferences
