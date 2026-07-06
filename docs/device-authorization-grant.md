# Device Authorization Grant (RFC 8628)

FerrisKey supports the OAuth 2.0 Device Authorization Grant for browserless clients (CLI tools, TV, IoT, and similar devices).

Flow diagram:

1. The device requests a `device_code` from the device authorization endpoint.
2. The device prompts the user with `user_code` and `verification_uri`.
3. A separate browser session approves/denies the request.
4. The device polls the token endpoint with `urn:ietf:params:oauth:grant-type:device_code` until success or error.

## 1) Device code initiation

```bash
curl -sS -X POST 'https://auth.example.com/realms/demo/protocol/openid-connect/auth/device' \
  -H 'Content-Type: application/x-www-form-urlencoded' \
  -d 'client_id=cli-client' \
  -d 'scope=openid profile email'
```

Response example:

```json
{
  "device_code": "019...",
  "user_code": "ABCD-EFGH",
  "verification_uri": "https://auth.example.com/realms/demo/device",
  "verification_uri_complete": "https://auth.example.com/realms/demo/device?user_code=ABCD-EFGH",
  "expires_in": 600,
  "interval": 5
}
```

- `device_code`: opaque value the device keeps and polls with.
- `user_code`: short code shown to the user (`XXXX-XXXX`).
- `verification_uri`: page where the user enters the code.
- `verification_uri_complete`: convenience URL already containing `user_code`.
- `expires_in`: seconds before the request expires.
- `interval`: minimum polling interval in seconds.

## 2) User approval (browser)

Open either:

- `verification_uri` and type the `user_code`, or
- `verification_uri_complete`.

Then authenticate and approve/deny the request.

## 3) Poll for tokens

```bash
curl -sS -X POST 'https://auth.example.com/realms/demo/protocol/openid-connect/token' \
  -H 'Content-Type: application/x-www-form-urlencoded' \
  -d 'grant_type=urn:ietf:params:oauth:grant-type:device_code' \
  -d 'client_id=cli-client' \
  -d 'device_code=019...'
```

`interval` should be respected to avoid `slow_down`. Reuse the same `client_id` used in step 1.

Common token polling responses:

- `200`: contains `access_token`/`id_token`/`refresh_token`.
- `400` with `authorization_pending`: still waiting for user approval.
- `400` with `slow_down`: poll less frequently.
- `400` with `expired_token`: the user code expired.
- `400` with `access_denied`: user denied the request.

## Grant type string

Use the exact RFC 8628 grant type value:

```text
urn:ietf:params:oauth:grant-type:device_code
```

## Webhook triggers

FerrisKey emits these webhook events for device flow states:

- `auth.device_flow.initiated`
- `auth.device_flow.denied`
- `auth.device_flow.expired`

## CLI use case

This grant is intended for CLI and other browserless clients. The upcoming FerrisKey CLI work is tracked in the Device Authorization Epic (issue [#1020](https://github.com/ferriskey/ferriskey/issues/1020)).

## Reference

- RFC 8628: https://datatracker.ietf.org/doc/html/rfc8628
