mod common;

use std::{collections::HashMap, fmt};

use axum_test::{TestResponse, TestServer};
use common::{
    CALLBACK, ClientSpec, SeededClient, SeededUser, TestApp, app, authorize, client_credentials,
    exchange_code, introspect, query_param, refresh, rt, sign_in_for_code,
};
use cucumber::{World, given, then, when};
use serde_json::Value;

const PASSWORD: &str = "Scenario-Passw0rd!";
const PKCE_VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
const PKCE_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

struct Reply {
    status: u16,
    body: Value,
    location: Option<String>,
}

impl Reply {
    fn from(response: TestResponse) -> Self {
        let location = response
            .maybe_header("location")
            .and_then(|value| value.to_str().ok().map(str::to_string));
        let body = serde_json::from_str(&response.text()).unwrap_or(Value::Null);
        Self {
            status: response.status_code().as_u16(),
            body,
            location,
        }
    }
}

#[derive(World)]
#[world(init = Self::new)]
struct OidcWorld {
    app: &'static TestApp,
    server: TestServer,
    clients: HashMap<String, SeededClient>,
    users: HashMap<String, SeededUser>,
    code: Option<(String, String)>,
    tokens: Option<Value>,
    rotated_refresh_token: Option<String>,
    reply: Option<Reply>,
}

impl fmt::Debug for OidcWorld {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OidcWorld")
            .field("clients", &self.clients.keys().collect::<Vec<_>>())
            .field("users", &self.users.keys().collect::<Vec<_>>())
            .field("tokens", &self.tokens)
            .field("reply_status", &self.reply.as_ref().map(|r| r.status))
            .field("reply_body", &self.reply.as_ref().map(|r| &r.body))
            .field("reply_location", &self.reply.as_ref().map(|r| &r.location))
            .finish()
    }
}

impl OidcWorld {
    async fn new() -> Self {
        let app = app();
        Self {
            app,
            server: app.server(),
            clients: HashMap::new(),
            users: HashMap::new(),
            code: None,
            tokens: None,
            rotated_refresh_token: None,
            reply: None,
        }
    }

    fn client(&self, alias: &str) -> &SeededClient {
        self.clients
            .get(alias)
            .unwrap_or_else(|| panic!("no application named {alias:?}"))
    }

    fn user(&self, alias: &str) -> &SeededUser {
        self.users
            .get(alias)
            .unwrap_or_else(|| panic!("no user named {alias:?}"))
    }

    fn reply(&self) -> &Reply {
        self.reply.as_ref().expect("no response recorded")
    }

    fn tokens(&self) -> &Value {
        self.tokens.as_ref().expect("no tokens issued yet")
    }

    fn token(&self, field: &str) -> String {
        self.tokens()[field]
            .as_str()
            .unwrap_or_else(|| panic!("{field} missing from {}", self.tokens()))
            .to_string()
    }

    async fn sign_in(&mut self, user: &str, client: &str, extra: &[(&str, &str)]) -> String {
        let user = self.user(user);
        let client = self.client(client);
        sign_in_for_code(
            &self.server,
            self.app,
            &client.client_id,
            &user.username,
            &user.password,
            extra,
        )
        .await
    }

    async fn sign_in_and_redeem(&mut self, user: &str, client: &str) {
        let code = self.sign_in(user, client, &[]).await;
        let response = exchange_code(&self.server, self.app, self.client(client), &code, &[]).await;
        let reply = Reply::from(response);
        assert_eq!(reply.status, 200, "code exchange: {}", reply.body);
        self.tokens = Some(reply.body.clone());
        self.code = Some((client.to_string(), code));
        self.reply = Some(reply);
    }
}

#[given(expr = "a confidential application {string}")]
async fn confidential_application(world: &mut OidcWorld, alias: String) {
    let client = world.app.client(ClientSpec::confidential()).await;
    world.clients.insert(alias, client);
}

#[given(expr = "a public application {string}")]
async fn public_application(world: &mut OidcWorld, alias: String) {
    let client = world.app.client(ClientSpec::public()).await;
    world.clients.insert(alias, client);
}

#[given(expr = "a confidential application {string} with a service account")]
async fn service_application(world: &mut OidcWorld, alias: String) {
    let client = world
        .app
        .client(ClientSpec::confidential().with_service_account())
        .await;
    world.app.bind_service_account(&world.server, &client).await;
    world.clients.insert(alias, client);
}

#[given(expr = "{string} is disabled")]
async fn application_disabled(world: &mut OidcWorld, alias: String) {
    world
        .app
        .set_client_enabled(world.client(&alias), false)
        .await;
}

#[given(expr = "{string} accepts {string} after logout")]
async fn post_logout_address(world: &mut OidcWorld, alias: String, uri: String) {
    world
        .app
        .add_post_logout_redirect_uri(world.client(&alias), &uri)
        .await;
}

#[given(expr = "{string} issues ID tokens that are already expired")]
async fn expired_id_tokens(world: &mut OidcWorld, alias: String) {
    world
        .app
        .set_id_token_lifetime(world.client(&alias), -60)
        .await;
}

#[given(expr = "a registered user {string}")]
async fn registered_user(world: &mut OidcWorld, alias: String) {
    let user = world.app.user(&world.server, PASSWORD).await;
    world.users.insert(alias, user);
}

#[given(expr = "{string} has signed in to {string}")]
async fn has_signed_in(world: &mut OidcWorld, user: String, client: String) {
    world.sign_in_and_redeem(&user, &client).await;
}

#[when(expr = "{string} signs in to {string}")]
async fn signs_in(world: &mut OidcWorld, user: String, client: String) {
    world.sign_in_and_redeem(&user, &client).await;
}

#[when(expr = "{string} signs in to {string} with a PKCE S256 challenge")]
async fn signs_in_with_pkce(world: &mut OidcWorld, user: String, client: String) {
    let code = world
        .sign_in(
            &user,
            &client,
            &[
                ("code_challenge", PKCE_CHALLENGE),
                ("code_challenge_method", "S256"),
            ],
        )
        .await;
    let response = exchange_code(
        &world.server,
        world.app,
        world.client(&client),
        &code,
        &[("code_verifier", PKCE_VERIFIER)],
    )
    .await;
    let reply = Reply::from(response);
    world.tokens = (reply.status == 200).then(|| reply.body.clone());
    world.reply = Some(reply);
}

#[when(expr = "{string} redeems the same code again")]
async fn redeems_again(world: &mut OidcWorld, client: String) {
    let (_, code) = world.code.clone().expect("no code redeemed yet");
    let response = exchange_code(&world.server, world.app, world.client(&client), &code, &[]).await;
    world.reply = Some(Reply::from(response));
}

#[given(expr = "{string} has refreshed its tokens once")]
async fn refreshed_once(world: &mut OidcWorld, client: String) {
    let previous = world.token("refresh_token");
    let seeded = world.client(&client);
    let response = refresh(
        &world.server,
        world.app,
        &seeded.client_id,
        seeded.secret.as_deref(),
        &previous,
    )
    .await;
    let reply = Reply::from(response);
    assert_eq!(reply.status, 200, "first refresh: {}", reply.body);
    world.tokens = Some(reply.body.clone());
    world.rotated_refresh_token = Some(previous);
}

#[when(expr = "{string} refreshes its tokens")]
async fn refreshes(world: &mut OidcWorld, client: String) {
    let token = world.token("refresh_token");
    let seeded = world.client(&client);
    let response = refresh(
        &world.server,
        world.app,
        &seeded.client_id,
        seeded.secret.as_deref(),
        &token,
    )
    .await;
    world.reply = Some(Reply::from(response));
}

#[when(expr = "{string} refreshes its tokens without its client secret")]
async fn refreshes_without_secret(world: &mut OidcWorld, client: String) {
    let token = world.token("refresh_token");
    let client_id = world.client(&client).client_id.clone();
    let response = refresh(&world.server, world.app, &client_id, None, &token).await;
    world.reply = Some(Reply::from(response));
}

#[when(expr = "{string} refreshes with the refresh token it already rotated")]
async fn refreshes_with_rotated(world: &mut OidcWorld, client: String) {
    let token = world
        .rotated_refresh_token
        .clone()
        .expect("no rotated token");
    let seeded = world.client(&client);
    let response = refresh(
        &world.server,
        world.app,
        &seeded.client_id,
        seeded.secret.as_deref(),
        &token,
    )
    .await;
    world.reply = Some(Reply::from(response));
}

#[when(expr = "{string} introspects the refresh token it already rotated")]
async fn introspects_rotated(world: &mut OidcWorld, client: String) {
    let token = world
        .rotated_refresh_token
        .clone()
        .expect("no rotated token");
    let body = introspect(&world.server, world.app, world.client(&client), &token).await;
    world.reply = Some(Reply {
        status: 200,
        body,
        location: None,
    });
}

#[when(expr = "{string} requests a token with its own credentials")]
async fn requests_client_credentials(world: &mut OidcWorld, client: String) {
    let response = client_credentials(&world.server, world.app, world.client(&client)).await;
    world.reply = Some(Reply::from(response));
}

#[when(
    expr = "an authorization request for {string} carries prompt {string} and redirect_uri {string}"
)]
async fn authorization_with_prompt(
    world: &mut OidcWorld,
    client: String,
    prompt: String,
    redirect_uri: String,
) {
    let response = world
        .server
        .get(&world.app.oidc("auth"))
        .add_query_param("response_type", "code")
        .add_query_param("client_id", &world.client(&client).client_id)
        .add_query_param("redirect_uri", &redirect_uri)
        .add_query_param("scope", "openid")
        .add_query_param("prompt", &prompt)
        .await;
    world.reply = Some(Reply::from(response));
}

#[when(expr = "an authorization request for {string} asks for response_type {string}")]
async fn authorization_with_response_type(
    world: &mut OidcWorld,
    client: String,
    response_type: String,
) {
    let response = world
        .server
        .get(&world.app.oidc("auth"))
        .add_query_param("response_type", &response_type)
        .add_query_param("client_id", &world.client(&client).client_id)
        .add_query_param("redirect_uri", CALLBACK)
        .add_query_param("scope", "openid")
        .await;
    world.reply = Some(Reply::from(response));
}

#[when(expr = "an authorization request for {string} is sent without PKCE")]
async fn authorization_without_pkce(world: &mut OidcWorld, client: String) {
    let client_id = world.client(&client).client_id.clone();
    let response = authorize(&world.server, world.app, &client_id, &[]).await;
    world.reply = Some(Reply::from(response));
}

#[when(expr = "{string} logs out of {string} towards {string} with state {string}")]
async fn logs_out(
    world: &mut OidcWorld,
    _user: String,
    client: String,
    uri: String,
    state: String,
) {
    let id_token = world.token("id_token");
    let response = world
        .server
        .get(&world.app.oidc("logout"))
        .add_query_param("id_token_hint", &id_token)
        .add_query_param("client_id", &world.client(&client).client_id)
        .add_query_param("post_logout_redirect_uri", &uri)
        .add_query_param("state", &state)
        .await;
    world.reply = Some(Reply::from(response));
}

#[then(expr = "{string} receives an ID token for {string}")]
async fn receives_id_token(world: &mut OidcWorld, client: String, user: String) {
    let reply = world.reply();
    assert_eq!(reply.status, 200, "token response: {}", reply.body);
    let id_token = world.token("id_token");
    let claims = common::decode_unverified(&id_token);
    assert_eq!(claims["sub"].as_str(), Some(world.user(&user).id.as_str()));
    assert_eq!(
        claims["azp"].as_str(),
        Some(world.client(&client).client_id.as_str())
    );
}

#[then(expr = "the token endpoint refuses with {string}")]
async fn token_endpoint_refuses(world: &mut OidcWorld, error: String) {
    let reply = world.reply();
    assert!(
        (400..500).contains(&reply.status),
        "expected a refusal, got {}: {}",
        reply.status,
        reply.body
    );
    assert_eq!(
        reply.body["error"].as_str(),
        Some(error.as_str()),
        "{}",
        reply.body
    );
}

#[then(expr = "the token endpoint issues an access token")]
async fn token_endpoint_issues(world: &mut OidcWorld) {
    let reply = world.reply();
    assert_eq!(reply.status, 200, "{}", reply.body);
    assert!(reply.body["access_token"].is_string(), "{}", reply.body);
}

#[then(expr = "the access token first issued to {string} is no longer active")]
async fn first_access_token_inactive(world: &mut OidcWorld, client: String) {
    let token = world.token("access_token");
    let body = introspect(&world.server, world.app, world.client(&client), &token).await;
    assert_eq!(body["active"], false, "{body}");
}

#[then(expr = "the token is reported inactive")]
async fn token_inactive(world: &mut OidcWorld) {
    assert_eq!(
        world.reply().body["active"],
        false,
        "{}",
        world.reply().body
    );
}

#[then(expr = "the browser is not sent to {string}")]
async fn not_sent_to(world: &mut OidcWorld, host: String) {
    if let Some(location) = world.reply().location.as_deref() {
        assert!(!location.contains(&host), "redirected to {location}");
    }
}

#[then(expr = "the browser is sent to {string} with state {string}")]
async fn sent_to_with_state(world: &mut OidcWorld, uri: String, state: String) {
    let reply = world.reply();
    let location = reply
        .location
        .as_deref()
        .unwrap_or_else(|| panic!("no redirect, got {}: {}", reply.status, reply.body));
    assert!(location.starts_with(&uri), "redirected to {location}");
    assert_eq!(
        query_param(location, "state").as_deref(),
        Some(state.as_str())
    );
}

#[then(expr = "the authorization request is refused with {string}")]
async fn authorization_refused(world: &mut OidcWorld, error: String) {
    let reply = world.reply();
    let location = reply
        .location
        .as_deref()
        .unwrap_or_else(|| panic!("no redirect, got {}: {}", reply.status, reply.body));
    assert_eq!(
        query_param(location, "error").as_deref(),
        Some(error.as_str()),
        "redirected to {location}"
    );
}

fn main() {
    if std::env::var_os("FERRISKEY_BDD").is_none() {
        eprintln!("oidc_bdd skipped: set FERRISKEY_BDD=1 with a reachable PostgreSQL to run it");
        return;
    }
    let wip = std::env::var_os("FERRISKEY_BDD_WIP").is_some();
    let features = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/features/oidc");

    rt().block_on(async {
        let _ = app();
        OidcWorld::cucumber()
            .fail_on_skipped()
            .filter_run_and_exit(features, move |_, _, scenario| {
                scenario.tags.iter().any(|tag| tag == "wip") == wip
            })
            .await;
    });
}
