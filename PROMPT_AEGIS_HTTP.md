# Prompt: Implement Aegis (Client Scope) HTTP Endpoints

## Objective

Implement the HTTP API layer for the **aegis** module (client scopes) in `api/`, following the Keycloak Admin REST API patterns and the existing FerrisKey codebase conventions.

The domain layer (`core/src/domain/aegis/`) and infrastructure layer (`core/src/infrastructure/aegis/`) are already implemented. You need to:
1. Wire the aegis services into `ApplicationService`
2. Create the HTTP handlers, validators, and router
3. Register the routes in the main server

---

## Step 1: Wire Aegis Services into ApplicationService

### 1.1 Add type aliases in `core/src/application/services.rs`

Add these type aliases alongside the existing ones:

```rust
type ClientScopeRepo = PostgresClientScopeRepository;
type ClientScopeAttributeRepo = PostgresClientScopeAttributeRepository;
type ProtocolMapperRepo = PostgresProtocolMapperRepository;
type ScopeMappingRepo = PostgresScopeMappingRepository;
```

Add the required imports from:
- `crate::infrastructure::aegis::repositories::client_scope_postgres_repository::PostgresClientScopeRepository`
- `crate::infrastructure::aegis::repositories::client_scope_attribute_postgres_repository::PostgresClientScopeAttributeRepository`
- `crate::infrastructure::aegis::repositories::protocol_mapper_postgres_repository::PostgresProtocolMapperRepository`
- `crate::infrastructure::aegis::repositories::scope_mapping_postgres_repository::PostgresScopeMappingRepository`

Add three new fields to the `ApplicationService` struct:

```rust
pub(crate) client_scope_service: ClientScopeServiceImpl<RealmRepo, UserRepo, ClientRepo, UserRoleRepo, ClientScopeRepo>,
pub(crate) protocol_mapper_service: ProtocolMapperServiceImpl<RealmRepo, UserRepo, ClientRepo, UserRoleRepo, ClientScopeRepo, ProtocolMapperRepo>,
pub(crate) scope_mapping_service: ScopeMappingServiceImpl<RealmRepo, UserRepo, ClientRepo, UserRoleRepo, ClientScopeRepo, ScopeMappingRepo>,
```

Add the corresponding imports for the service types:
- `crate::domain::aegis::services::client_scope_service::ClientScopeServiceImpl`
- `crate::domain::aegis::services::protocol_mapper_service::ProtocolMapperServiceImpl`
- `crate::domain::aegis::services::scope_mapping_service::ScopeMappingServiceImpl`

### 1.2 Instantiate repositories and services in `core/src/application/mod.rs`

In the `create_service` function, add repository instantiation (after the existing repos):

```rust
let client_scope = Arc::new(PostgresClientScopeRepository::new(postgres.get_db()));
let client_scope_attr = Arc::new(PostgresClientScopeAttributeRepository::new(postgres.get_db()));
let protocol_mapper = Arc::new(PostgresProtocolMapperRepository::new(postgres.get_db()));
let scope_mapping = Arc::new(PostgresScopeMappingRepository::new(postgres.get_db()));
```

Add the service fields to the `ApplicationService` construction:

```rust
client_scope_service: ClientScopeServiceImpl::new(
    realm.clone(),
    client_scope.clone(),
    policy.clone(),
),
protocol_mapper_service: ProtocolMapperServiceImpl::new(
    realm.clone(),
    client_scope.clone(),
    protocol_mapper.clone(),
    policy.clone(),
),
scope_mapping_service: ScopeMappingServiceImpl::new(
    realm.clone(),
    client_scope.clone(),
    scope_mapping.clone(),
    policy.clone(),
),
```

Add the required imports for the aegis infrastructure and services.

### 1.3 Create trait delegation files in `core/src/application/`

Create `core/src/application/aegis.rs` following the pattern from `core/src/application/client/mod.rs`.

This file implements the three service traits for `ApplicationService` by delegating to the inner services:

```rust
impl ClientScopeService for ApplicationService { /* delegate to self.client_scope_service */ }
impl ProtocolMapperService for ApplicationService { /* delegate to self.protocol_mapper_service */ }
impl ScopeMappingService for ApplicationService { /* delegate to self.scope_mapping_service */ }
```

Register the module in `core/src/application/mod.rs`:
```rust
pub mod aegis;
```

---

## Step 2: Create HTTP Endpoints

### 2.1 Endpoints to implement

Following the Keycloak API, implement these endpoints:

#### Client Scope CRUD (`/realms/{realm_name}/client-scopes`)

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| `GET` | `/realms/{realm_name}/client-scopes` | `get_client_scopes` | List all client scopes in a realm |
| `POST` | `/realms/{realm_name}/client-scopes` | `create_client_scope` | Create a new client scope |
| `GET` | `/realms/{realm_name}/client-scopes/{scope_id}` | `get_client_scope` | Get a specific client scope |
| `PATCH` | `/realms/{realm_name}/client-scopes/{scope_id}` | `update_client_scope` | Update a client scope |
| `DELETE` | `/realms/{realm_name}/client-scopes/{scope_id}` | `delete_client_scope` | Delete a client scope |

#### Protocol Mappers (`/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers`)

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| `POST` | `/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers` | `create_protocol_mapper` | Create a protocol mapper |
| `PATCH` | `/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers/{mapper_id}` | `update_protocol_mapper` | Update a protocol mapper |
| `DELETE` | `/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers/{mapper_id}` | `delete_protocol_mapper` | Delete a protocol mapper |

#### Scope Mappings (Client <-> Scope associations)

These go under the **client** routes:

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| `PUT` | `/realms/{realm_name}/clients/{client_id}/default-client-scopes/{scope_id}` | `assign_default_scope` | Assign a default scope to a client |
| `DELETE` | `/realms/{realm_name}/clients/{client_id}/default-client-scopes/{scope_id}` | `unassign_default_scope` | Remove a default scope from a client |
| `PUT` | `/realms/{realm_name}/clients/{client_id}/optional-client-scopes/{scope_id}` | `assign_optional_scope` | Assign an optional scope to a client |
| `DELETE` | `/realms/{realm_name}/clients/{client_id}/optional-client-scopes/{scope_id}` | `unassign_optional_scope` | Remove an optional scope from a client |

### 2.2 File structure to create

```
api/src/application/http/aegis/
├── handlers.rs              # pub mod declarations for all handler modules
├── router.rs                # Routes + OpenApi doc struct
├── validators.rs            # Request body validators
└── handlers/
    ├── create_client_scope.rs
    ├── get_client_scopes.rs
    ├── get_client_scope.rs
    ├── update_client_scope.rs
    ├── delete_client_scope.rs
    ├── create_protocol_mapper.rs
    ├── update_protocol_mapper.rs
    ├── delete_protocol_mapper.rs
    ├── assign_default_scope.rs
    ├── unassign_default_scope.rs
    ├── assign_optional_scope.rs
    └── unassign_optional_scope.rs
```

### 2.3 Validators (`validators.rs`)

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateClientScopeValidator {
    #[validate(length(min = 1, message = "name is required"))]
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[validate(length(min = 1, message = "protocol is required"))]
    #[serde(default)]
    pub protocol: String,
    #[serde(default)]
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateClientScopeValidator {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub is_default: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateProtocolMapperValidator {
    #[validate(length(min = 1, message = "name is required"))]
    #[serde(default)]
    pub name: String,
    #[validate(length(min = 1, message = "mapper_type is required"))]
    #[serde(default)]
    pub mapper_type: String,
    #[serde(default = "default_config")]
    pub config: serde_json::Value,
}

fn default_config() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateProtocolMapperValidator {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub mapper_type: Option<String>,
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}
```

### 2.4 Handler pattern

Every handler follows this exact pattern (use `client` handlers as reference):

```rust
use crate::application::http::{
    aegis::validators::CreateClientScopeValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};
use axum::{Extension, extract::{Path, State}};
use ferriskey_core::domain::aegis::entities::ClientScope;
use ferriskey_core::domain::aegis::ports::ClientScopeService;
use ferriskey_core::domain::aegis::value_objects::CreateClientScopeInput;
use ferriskey_core::domain::authentication::value_objects::Identity;

#[utoipa::path(
    post,
    path = "",
    summary = "Create a new client scope",
    description = "Creates a new client scope within the specified realm.",
    responses(
        (status = 201, body = ClientScope, description = "Client scope created successfully"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    tag = "client-scope",
    request_body = CreateClientScopeValidator,
)]
pub async fn create_client_scope(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateClientScopeValidator>,
) -> Result<Response<ClientScope>, ApiError> {
    let scope = state
        .service
        .create_client_scope(
            identity,
            CreateClientScopeInput {
                realm_name,
                name: payload.name,
                description: payload.description,
                protocol: payload.protocol,
                is_default: payload.is_default,
            },
        )
        .await?;

    Ok(Response::Created(scope))
}
```

**Key rules for handlers:**
- Service calls use `state.service.<method>()` (trait dispatch via `ApplicationService`)
- Path extraction: single param = `Path(realm_name): Path<String>`, multiple = `Path((realm_name, scope_id)): Path<(String, Uuid)>`
- Always use `Extension(identity): Extension<Identity>` for auth
- Use `ValidateJson` for request bodies, omit for GET/DELETE without body
- Use `?` operator for error propagation (automatic `CoreError` -> `ApiError` conversion)
- Return `Response::Created` for POST, `Response::OK` for GET/PATCH/PUT/DELETE

**Response wrappers for list endpoints:**
```rust
#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ClientScopesResponse {
    pub data: Vec<ClientScope>,
}
```

**Delete response:**
```rust
#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct DeleteClientScopeResponse {
    pub message: String,
}
```

**Scope mapping handlers (assign/unassign):**
- Path: `Path((realm_name, client_id, scope_id)): Path<(String, Uuid, Uuid)>`
- Assign default: calls `state.service.assign_scope_to_client()` with `is_default: true, is_optional: false`
- Assign optional: calls `state.service.assign_scope_to_client()` with `is_default: false, is_optional: true`
- Unassign: calls `state.service.unassign_scope_from_client()`
- Assign returns `Response::OK(mapping)`, unassign returns `Response::OK(())`

### 2.5 Router (`router.rs`)

Follow the pattern from `api/src/application/http/client/router.rs`:

```rust
use axum::{Router, middleware, routing::{delete, get, patch, post, put}};
use utoipa::OpenApi;
use super::handlers::{/* import all handlers and __path_ functions */};
use crate::application::{auth::auth, http::server::app_state::AppState};

#[derive(OpenApi)]
#[openapi(
    paths(
        create_client_scope,
        get_client_scopes,
        get_client_scope,
        update_client_scope,
        delete_client_scope,
        create_protocol_mapper,
        update_protocol_mapper,
        delete_protocol_mapper,
        assign_default_scope,
        unassign_default_scope,
        assign_optional_scope,
        unassign_optional_scope,
    ),
    tags(
        (name = "client-scope", description = "Client scope management")
    )
)]
pub struct AegisApiDoc;

pub fn aegis_routes(state: AppState) -> Router<AppState> {
    let root = &state.args.server.root_path;

    Router::new()
        // Client Scope CRUD
        .route(
            &format!("{root}/realms/{{realm_name}}/client-scopes"),
            get(get_client_scopes).post(create_client_scope),
        )
        .route(
            &format!("{root}/realms/{{realm_name}}/client-scopes/{{scope_id}}"),
            get(get_client_scope).patch(update_client_scope).delete(delete_client_scope),
        )
        // Protocol Mappers
        .route(
            &format!("{root}/realms/{{realm_name}}/client-scopes/{{scope_id}}/protocol-mappers"),
            post(create_protocol_mapper),
        )
        .route(
            &format!("{root}/realms/{{realm_name}}/client-scopes/{{scope_id}}/protocol-mappers/{{mapper_id}}"),
            patch(update_protocol_mapper).delete(delete_protocol_mapper),
        )
        // Scope Mappings (Client <-> Scope)
        .route(
            &format!("{root}/realms/{{realm_name}}/clients/{{client_id}}/default-client-scopes/{{scope_id}}"),
            put(assign_default_scope).delete(unassign_default_scope),
        )
        .route(
            &format!("{root}/realms/{{realm_name}}/clients/{{client_id}}/optional-client-scopes/{{scope_id}}"),
            put(assign_optional_scope).delete(unassign_optional_scope),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
```

### 2.6 Module declarations (`handlers.rs`)

```rust
pub mod create_client_scope;
pub mod get_client_scopes;
pub mod get_client_scope;
pub mod update_client_scope;
pub mod delete_client_scope;
pub mod create_protocol_mapper;
pub mod update_protocol_mapper;
pub mod delete_protocol_mapper;
pub mod assign_default_scope;
pub mod unassign_default_scope;
pub mod assign_optional_scope;
pub mod unassign_optional_scope;
```

---

## Step 3: Register in Main Server

### 3.1 Add module declaration in `api/src/application/http.rs`

```rust
pub mod aegis;
```

### 3.2 Add routes in `api/src/application/http/server/http_server.rs`

Import:
```rust
use crate::application::http::aegis::router::aegis_routes;
```

Add to the router chain (after `.merge(abyss_routes(state.clone()))`):
```rust
.merge(aegis_routes(state.clone()))
```

### 3.3 Register OpenAPI in `api/src/application/http/server/openapi.rs`

Import:
```rust
use crate::application::http::aegis::router::AegisApiDoc;
```

Add to the `nest` list:
```rust
(path = "/realms/{realm_name}/client-scopes", api = AegisApiDoc),
```

---

## Domain Types Reference

These types already exist and should be used as-is:

### Entities (`core/src/domain/aegis/entities.rs`)
- `ClientScope` - id, realm_id, name, description, protocol, is_default, attributes, protocol_mappers, created_at, updated_at
- `ProtocolMapper` - id, client_scope_id, name, mapper_type, config (serde_json::Value), created_at
- `ClientScopeMapping` - client_id, scope_id, is_default, is_optional

### Service Input DTOs (`core/src/domain/aegis/value_objects.rs`)
- `CreateClientScopeInput` - realm_name, name, description, protocol, is_default
- `GetClientScopeInput` - realm_name, scope_id
- `GetClientScopesInput` - realm_name
- `UpdateClientScopeInput` - realm_name, scope_id, payload (UpdateClientScopeRequest)
- `DeleteClientScopeInput` - realm_name, scope_id
- `AssignClientScopeInput` - realm_name, client_id, scope_id, is_default, is_optional
- `UnassignClientScopeInput` - realm_name, client_id, scope_id
- `CreateProtocolMapperInput` - realm_name, scope_id, name, mapper_type, config
- `UpdateProtocolMapperInput` - realm_name, scope_id, mapper_id, payload (UpdateProtocolMapperRequest)
- `DeleteProtocolMapperInput` - realm_name, scope_id, mapper_id

### Service Traits (`core/src/domain/aegis/ports.rs`)
- `ClientScopeService` - create_client_scope, get_client_scope, get_client_scopes, update_client_scope, delete_client_scope
- `ProtocolMapperService` - create_protocol_mapper, update_protocol_mapper, delete_protocol_mapper
- `ScopeMappingService` - assign_scope_to_client, unassign_scope_from_client

---

## Verification

After implementation, run:
```bash
cargo check -p ferriskey-api
cargo clippy -p ferriskey-api
cargo fmt -- --check
```

All three commands must pass without errors.
