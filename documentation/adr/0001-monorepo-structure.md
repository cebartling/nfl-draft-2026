# ADR 0001: Monorepo Structure with Separate Backend and Frontend

## Status

Accepted

## Context

We needed to decide on the repository structure for a full-stack application with a Rust backend and SvelteKit frontend. The options were:

1. **Monorepo**: Single repository containing both backend and frontend
2. **Polyrepo**: Separate repositories for backend and frontend
3. **Integrated**: Frontend embedded within backend directory structure

The project requires:

- Coordinated development between frontend and backend
- Shared documentation and planning
- Type safety across the stack (shared type definitions conceptually)
- Independent deployment capabilities
- Clear separation of concerns

## Decision

We will use a **monorepo structure** with clearly separated `back-end/` and `front-end/` directories at the root level.

Repository structure:

```
nfl-draft-2026/
├── back-end/              # Rust backend (Cargo workspace)
├── front-end/             # SvelteKit application
├── documentation/         # Shared documentation
├── docker-compose.yml     # Shared infrastructure
└── README.md              # Project-wide documentation
```

## Consequences

### Positive

- **Simplified Coordination**: Frontend and backend changes can be committed together, ensuring API contracts stay in sync
- **Shared Documentation**: Architecture decisions, API contracts, and planning documents are co-located
- **Single CI/CD Pipeline**: Ability to run both frontend and backend tests in a single workflow
- **Easier Onboarding**: New developers clone one repository and have the entire application
- **Atomic Changes**: API changes and frontend updates can be reviewed together in a single PR
- **Shared Infrastructure**: Docker Compose configuration for PostgreSQL is shared between both stacks

### Negative

- **Larger Repository Size**: The repository contains both Rust and Node.js dependencies
- **Mixed Tooling**: Developers need both Rust and Node.js toolchains installed
- **Build Complexity**: CI/CD needs to handle two different build systems
- **Potential Merge Conflicts**: More active files can lead to more conflicts

### Neutral

- **Independent Deployment**: Each directory can still be deployed independently
- **Technology Independence**: Frontend and backend remain decoupled despite being in the same repo
- **Clear Boundaries**: Directory structure makes ownership and responsibilities obvious

## Notes

- The monorepo does NOT use workspace tools like Nx, Turborepo, or Bazel - it's a simple directory structure
- Each stack (backend/frontend) maintains its own build system (Cargo/npm)
- Docker Compose at the root manages shared infrastructure (PostgreSQL, pgAdmin)
- Documentation directory contains architecture decisions that affect both stacks
