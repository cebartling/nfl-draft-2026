# OpenAPI Documentation

## Overview

The NFL Draft Simulator API now includes full OpenAPI 3.0 documentation using **utoipa** and **Swagger UI**.

## Accessing the Documentation

Once the server is running, you can access:

- **Swagger UI**: http://localhost:8000/swagger-ui/
- **OpenAPI JSON**: http://localhost:8000/api-docs/openapi.json

## What's Documented

### Endpoints

**Health**
- `GET /health` - Health check endpoint

**Teams** (`/api/v1/teams`)
- `GET /api/v1/teams` - List all teams
- `GET /api/v1/teams/{id}` - Get team by ID
- `POST /api/v1/teams` - Create a new team

**Players** (`/api/v1/players`)
- `GET /api/v1/players` - List all players
- `GET /api/v1/players/{id}` - Get player by ID
- `POST /api/v1/players` - Create a new player

**Drafts** (`/api/v1/drafts`)
- `POST /api/v1/drafts` - Create a new draft
- `GET /api/v1/drafts` - List all drafts
- `GET /api/v1/drafts/{id}` - Get draft by ID
- `POST /api/v1/drafts/{id}/initialize` - Initialize draft picks
- `GET /api/v1/drafts/{id}/picks` - Get all picks for a draft
- `GET /api/v1/drafts/{id}/picks/next` - Get next available pick
- `GET /api/v1/drafts/{id}/picks/available` - Get all available picks
- `POST /api/v1/drafts/{id}/start` - Start a draft
- `POST /api/v1/drafts/{id}/pause` - Pause a draft
- `POST /api/v1/drafts/{id}/complete` - Complete a draft

**Picks** (`/api/v1/picks`)
- `POST /api/v1/picks/{id}/make` - Make a draft pick

### Schemas

All request and response types are fully documented with their fields and types:

**Domain Models:**
- `Conference` - NFL conference (AFC/NFC)
- `Division` - NFL division (AFC East, NFC West, etc.)
- `Position` - Player positions (QB, RB, WR, etc.)
- `DraftStatus` - Draft lifecycle status

**Request Types:**
- `CreateTeamRequest`
- `CreatePlayerRequest`
- `CreateDraftRequest`
- `MakePickRequest`

**Response Types:**
- `TeamResponse`
- `PlayerResponse`
- `DraftResponse`
- `DraftPickResponse`

## Implementation Details

### Dependencies

```toml
utoipa = { version = "5.3", features = ["axum_extras", "chrono", "uuid"] }
utoipa-swagger-ui = { version = "9.0", features = ["axum"] }
```

### Code Annotations

All handlers are annotated with `#[utoipa::path]`:

```rust
#[utoipa::path(
    get,
    path = "/api/v1/teams/{id}",
    responses(
        (status = 200, description = "Team found", body = TeamResponse),
        (status = 404, description = "Team not found")
    ),
    params(
        ("id" = Uuid, Path, description = "Team ID")
    ),
    tag = "teams"
)]
pub async fn get_team(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<TeamResponse>> {
    // handler implementation
}
```

All request/response types have `#[derive(ToSchema)]`:

```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct TeamResponse {
    pub id: Uuid,
    pub name: String,
    pub abbreviation: String,
    pub city: String,
    pub conference: Conference,
    pub division: Division,
}
```

### OpenAPI Configuration

The OpenAPI spec is defined in `crates/api/src/openapi.rs`:

```rust
#[derive(OpenApi)]
#[openapi(
    info(
        title = "NFL Draft Simulator API",
        version = "0.1.0",
        description = "...",
    ),
    paths(
        health::health_check,
        teams::list_teams,
        teams::get_team,
        // ... all other paths
    ),
    components(
        schemas(
            Conference,
            Division,
            // ... all other schemas
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "teams", description = "NFL team management"),
        // ... all other tags
    )
)]
pub struct ApiDoc;
```

### Router Integration

Swagger UI is integrated into the main router in `crates/api/src/routes/mod.rs`:

```rust
// Create stateful routes
let stateful_router = Router::new()
    .route("/health", get(handlers::health::health_check))
    .nest("/api/v1", api_routes)
    .with_state(state);

// Swagger UI router (stateless)
let swagger_router: Router = SwaggerUi::new("/swagger-ui")
    .url("/api-docs/openapi.json", ApiDoc::openapi())
    .into();

// Merge routers and add layers
stateful_router
    .merge(swagger_router)
    .layer(cors)
    .layer(TraceLayer::new_for_http())
```

## Testing

To test the implementation:

1. Start the server:
   ```bash
   cargo run --package api
   ```

2. Access Swagger UI in your browser:
   ```
   http://localhost:8000/swagger-ui/
   ```

3. View the OpenAPI JSON spec:
   ```bash
   curl http://localhost:8000/api-docs/openapi.json | jq .
   ```

## Benefits

1. **Interactive Documentation**: Developers can explore and test the API directly from the browser
2. **Type Safety**: Schemas are generated from Rust types, ensuring they stay in sync
3. **Auto-Generated**: No need to manually write OpenAPI specs
4. **Standards Compliant**: Full OpenAPI 3.0 specification
5. **Client Generation**: The OpenAPI spec can be used to generate client libraries in any language

## Adding New Endpoints

When adding new endpoints:

1. Add `#[derive(ToSchema)]` to request/response types
2. Add `#[utoipa::path]` annotation to the handler
3. Add the handler to the `paths()` list in `ApiDoc`
4. Add any new types to the `components/schemas()` list

Example:

```rust
// 1. Annotate types
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MyRequest {
    pub field: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MyResponse {
    pub result: String,
}

// 2. Annotate handler
#[utoipa::path(
    post,
    path = "/api/v1/my-endpoint",
    request_body = MyRequest,
    responses(
        (status = 200, description = "Success", body = MyResponse)
    ),
    tag = "my-tag"
)]
pub async fn my_handler(
    State(state): State<AppState>,
    Json(req): Json<MyRequest>,
) -> ApiResult<Json<MyResponse>> {
    // implementation
}

// 3 & 4. Update ApiDoc in openapi.rs
paths(
    // ... existing paths
    my_module::my_handler,
),
components(
    schemas(
        // ... existing schemas
        my_module::MyRequest,
        my_module::MyResponse,
    )
)
```
