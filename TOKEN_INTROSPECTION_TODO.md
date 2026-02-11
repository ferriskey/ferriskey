# Token Introspection Implementation Checklist

Endpoint: `POST /realms/{realm_name}/protocol/openid-connect/token/introspect`

- [x] Add access-token persistence for revocation/opaque support (DB migration + SeaORM entity + repository + wiring)
- [x] Persist newly issued access tokens into store (on token mint)
- [x] Add core service method for introspection (client authz + access/refresh validation)
- [x] Add API handler + route + OpenAPI docs
- [x] Implement client authentication methods: `client_secret_basic`, `client_secret_post`
- [x] Enforce authorization: caller must have `introspect` scope (implemented as service-account role named `introspect`)
- [x] Add tests (auth parsing + scope manager)
- [x] Run `cargo test` / build
