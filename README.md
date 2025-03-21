# FerrisKey - Keycloak Light in Rust

## Project goals

FerrisKey is a lightweight, modular identity management and authentication system designed to offer a simplified alternative to Keycloak, focusing on the essential features of user authentication, authorization and management.

### Main objectives:

- Provide a lightweight OAuth2/OIDC server written in GB.
- Authentication and user management with PostgreSQL.
- JWT generation and validation.
- Support for essential OAuth2 flows (Authorization Code, Client Credentials, Refresh Token).
- Administration via a REST API and a React web interface.
- Cloud-native deployment on Kubernetes.



### API Endpoints


#### Endpoints for Realms management

- [ ] POST `/realms` - Create new realm
- [ ] GET `/realms` - List realms
- [ ] GET `/realms/{realm}` - Realm detail
- [ ] DELETE `/realms/{realm}` - Delete a realm

#### Endpoint for manage oauth2 clients

- [ ] POST `/realms/{realm}/clients` - Creating an OAuth2 client in a realm
- [ ] GET `/realms/{realm}/clients` - List of OAuth2 clients for a realm
- [ ] GET `/realms/{realm}/clients/{clientId}` - Details of an OAuth2 client
- [ ] DELETE `/realms/{realm}/clients/{clientId}` - Removing an OAuth2 client

#### Authentication and User Endpoints

- [ ] POST `/realms/{realm}/auth/signup` - New user registration
- [ ] POST `/realms/{realm}/auth/login` - Connecting and generating a JWT
- [ ] POST `/realms/{realm}/auth/refresh` - Token refresh
- [ ] POST `/realms/{realm}/auth/logout` - Log off
- [ ] GET `/realms/{realm}/users/{id}` - Retrieving user information
- [ ] PATCH `/realms/{realm}/users/{id}` - User profile update
- [ ] DELETE `/realms/{realm}/users/{id}` - Deleting a user

#### OAuth2 Endpoints

- [ ] POST `/realms/{realm}/protocol/openid-connect/token` - Exchange an authentication code for a JWT token
- [ ] GET `/realms/{realm}/protocol/openid-connect/introspect` - OAuth2 token validation
- [ ] GET `/realms/{realm}/protocol/openid-connect/jwks` - Public key recovery for JWT validation
- [ ] GET `/realms/{realm}/protocol/openid-connect/userinfo` - Authenticated user information retrieval