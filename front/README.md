# Ferriskey - Webapp (React)

## API Client Generation
This project uses `typed-openapi` to automatically generate TypeScript types and TanStack Query hooks from an OpenAPI specification. This ensures type safety and keeps the frontend in sync with the backend API.

### Prerequisites

Make sure you habe the OpenAPI specification file available (you can go to address API with `/swagger-ui` and downlaod the OpenAPI Document).

### Generating API Client and Types
To generate the API client and TanStack Query hooks, run:

```bash
pnpm typed-openapi openapi.yaml -o src/api/api.client.ts --tanstack=api.tanstack.ts && pnpm prettier --write src/api/api.client.ts src/api/api.tanstack.ts
```

**Command Breakdown:**
- `openapi.yaml`: Path to your OpenAPI specification file
- `-o src/api/api.client.ts`: Output path for the generated TypeScript types and schemas
- `--tanstack=api.tanstack.ts`: Generates TanStack Query hooks in the specified file
- `pnpm prettier --write`: Formats the generated files to match  project Prettier config

**Generated Files**
This command generates two main files: `src/api/api.client.ts`
Contains:
- **Schemas namespace**: All TypeScript types corresponding to your API models
- **Endpoints namespace**: Type definitions for all API endpoints
- **ApiClient class**: Base client for making HTTP requests
- **Helper types**: Utility types for request/response handling

Example usage:
```ts
import { Schemas } from '@/api/api.client'

// Use generated types
type User = Schemas.User
type CreateUserRequest = Schemas.CreateUserValidator
```
