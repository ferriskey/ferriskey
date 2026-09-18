# Translation glossary

The terms below carry domain meaning. Rendering one of them two ways across two files makes the
interface incoherent for a reader who cannot fall back on the English, so they are decided once here
and applied everywhere.

**Status: proposed, pending native review.** The Simplified Chinese column was produced without a
native speaker. Terms marked ⚠ are the ones where a reviewer's judgement matters most. Do not treat
this file as settled.

## Never translated

These are product names, protocol names or wire identifiers. Translating them breaks a search, a
standard, or stored data.

| Term | Why |
|---|---|
| FerrisKey | Product name |
| Trident, Seawatch, Compass, Abyss | FerrisKey module names — they are proper nouns, not descriptions |
| Google, GitHub, GitLab, Microsoft, Apple, Facebook, Discord, X (Twitter), LinkedIn, Okta | Third-party brands |
| OIDC, OAuth2, SAML, LDAP, TOTP, WebAuthn, PKCE, JWT, SMTP | Protocol and standard names |
| `snake_case` enum values, permission names, error `reason` codes | Contract with the API, visible in tokens and logs |
| Header names, URL examples, query-syntax hints | Technical examples — translating makes them wrong |

## Core IAM vocabulary

| English | 简体中文 | Note |
|---|---|---|
| realm | 域 | The tenant boundary. **Not** 领域 (a field of study) nor 王国. ⚠ Keycloak's Chinese UI uses 域, which migrating users will recognise. |
| client | 客户端 | An application, not a person. Distinct from "customer". |
| client scope | 客户端作用域 | |
| scope | 作用域 | The OAuth sense, not "extent of work". |
| credential | 凭据 | **Not** 证书 (certificate) — a credential here may be a password or a passkey. ⚠ |
| claim | 声明 | The JWT sense. |
| role | 角色 | |
| permission | 权限 | |
| identity provider | 身份提供商 | |
| user federation | 用户联合 | |
| mapper | 映射器 | |
| required action | 必需操作 | Something the user must complete before signing in. ⚠ |
| passkey | 通行密钥 | |
| magic link | 魔法链接 | ⚠ Widely used, but a reviewer may prefer 免密登录链接 ("passwordless sign-in link"), which explains rather than transliterates. |
| webhook | Webhook | Left in English; the transliteration is not established. |
| trigger | 触发器 | |
| delivery | 投递 | The webhook sense: an attempt to call an endpoint. |
| session | 会话 | |
| sign in / log in | 登录 | One verb for both. The English is inconsistent; the Chinese should not be. |
| sign out | 退出登录 | |
| organization | 组织 | |
| group | 组 | |
| member | 成员 | |
| attribute | 属性 | |
| redirect URI | 重定向 URI | |
| web origin | Web 来源 | |
| audit / security event | 安全事件 | |
| authentication flow | 认证流程 | |
| maintenance mode | 维护模式 | |

## Audience register

Two audiences, two registers. The catalogues must not be harmonised into one voice.

**`auth.yaml` — end users with no account.** They do not know what a realm, a client or a credential
is. Prefer plain wording over the table above wherever the English already avoids jargon. Where the
English *does* leak jargon — "device code charset", "WebAuthn is not supported", "Invalid OTP code" —
translate it faithfully and flag it: the fix is a copy change in English first.

**Every other catalogue — administrators.** The table above applies directly. An administrator
configuring authentication needs precision more than approachability.

## Terms whose English is itself unclear

Eight webhook trigger descriptions were reported by the extraction as ambiguous *in English*. A
translator cannot resolve them, and a confident translation would invent a meaning. They must be
answered by a maintainer before their Chinese is trusted:

| Trigger | Problem |
|---|---|
| `user.credentials.deleted` | "A user credentials have been deleted." Broken grammar, and ambiguous: all credentials of a user, or one? The label reads as if the user is the actor. |
| `auth.reset_password` | Does not say by whom. Self-service reset or admin-forced changes the verb in Chinese. |
| `client.maintenance.enabled` / `.disabled` | What maintenance mode actually does — reject authentication? read-only? — is FerrisKey-specific and absent from the copy. |
| `role.permission.updated` | With a bitmask permission model, one bit or the whole mask? |
| `auth.device_flow.denied` | Denied by the user refusing, or by policy? Different verbs. |
| `user.bulk_deleted` | Label is singular, description is plural. They disagree. |
| `web_origin.created` | Says "registered on a client" where the structurally identical `redirect_uri.created` says "has been created". |
| `client.saml_config.updated` | Needs confirming FerrisKey is the IdP and the client the SP, otherwise the translation inverts the roles. |

Until these are answered, their Chinese follows the English literally, defect included.

## Mechanical rules

- **Keys, `{{placeholders}}` and `<tags>` are contract.** Never translate, rename, reorder or drop
  them. `<em>` must survive as `<em>`; `{{count}}` must survive as `{{count}}`.
- **A placeholder may move inside the sentence** — that is the point of having it.
- **Chinese has one plural category.** A key with `_one` and `_other` in English gets **only
  `_other`** in Chinese. Adding `_one` fails the parity job, and so does omitting `_other`.
- **No spaces around CJK text**, except around Latin words, numbers and placeholders embedded in it.
- **Punctuation follows the target language**: `，` `。` `、` rather than `,` `.` `,` in Chinese
  sentences. Keep ASCII punctuation inside code, URLs and identifiers.
- **Enumerations use `、`**, not `, `. Where the list is built in code rather than in the catalogue,
  the separator is still English — that is a known gap, tracked on the epic.
