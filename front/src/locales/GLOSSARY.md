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
| web origin | Web 源 | The CORS sense. **Not** 来源, which is reserved below — the two appear on adjacent screens and meant the same word before this line existed. |
| origin (audit) | 来源 | Where a security event came from. A different concept from `web origin`, and the console renders both. |
| revoke | 撤销 | Sessions, tokens, consents. **Not** 吊销 (which reads as a punitive withdrawal of a licence) and never 注销 — 注销账号 means *delete the account*. One verb everywhere: a user and an administrator must be told the same action by the same word. |
| panel | 面板 | |
| portal | 门户 | |
| layout | 布局 | |
| theme | 主题 | |
| service provider | 服务提供商 | The SAML sense. |
| protocol mapper | 协议映射器 | |
| token | 令牌 | access / refresh / ID / temporary → 访问令牌 / 刷新令牌 / ID 令牌 / 临时令牌. **`ID 令牌`, not 身份令牌**, even where the English says "identity token". |
| client secret | 客户端密钥 | ⚠ "secret" and "signing key" both fall to 密钥 in natural Chinese, collapsing a distinction English keeps. Qualify: 客户端密钥 vs 签名密钥. |
| signing key | 签名密钥 | |
| grant | 授权类型 | ⚠ The OAuth sense. One rendering everywhere — `direct access grants` → 直接访问授权类型, `device authorization grant` → 设备授权类型. Reported as a real three-way drift on #1371; do not optimise per sentence. |
| flow | 流程 | authorization code flow → 授权码流程, device flow → 设备流程 |
| assertion | 断言 | The SAML sense. |
| service account | 服务账户 | |
| single sign-on | 单点登录 | |
| lifetime | 有效期 | |
| confidential / public | 机密 / 公开 | The client type. |
| whitelist | 白名单 | |
| application | 应用 | The CIAM panel's word for a client. Not 应用程序 — badge and column width. |
| identity | 身份 | The CIAM panel's word for a user. |
| account | 账户 | English contrasts *account* and *user*; collapsing both to 用户 loses "Accounts holding this role". Not 账号, not 帐户. |
| customer | 客户 | ⚠ One character from 客户端 (`client`). Deliberate, and fragile. |
| sign-up / registration | 注册 | |
| journal | 事件记录 | ⚠ The four activity lists. **Not** 日志, which is taken by "Logs & events" → 日志与事件 and would make two adjacent screens share a name. |
| endpoint | 端点 | |
| issuer | 颁发者 | |
| callback URL | 回调 URL | |
| pending action | 待处理操作 | The console's wording for a `required action` (必需操作). Two different English phrases for one concept; keep both renderings until the English is unified. |
| audience | 受众 | The OIDC `aud`. |
| membership | 成员关系 | |
| alias | 别名 | |
| lockout | 锁定 | |
| second factor / MFA | 第二因素 / 多因素认证 | |
| builder | 编辑器 | The drag-and-drop editors for emails and portal pages. Had no precedent across 17 catalogues; two lots needed it at once. |
| template | 模板 | |
| variable | 变量 | The `{{user.first_name}}` kind. |
| assignment / unassign | 分配 / 解除分配 | |
| transactional email | 事务性邮件 | |
| audit / security event | 安全事件 | |
| authentication flow | 认证流程 | |
| maintenance mode | 维护模式 | |

## Audience register

Two audiences, two registers. The catalogues must not be harmonised into one voice.

**`auth.yaml` — end users with no account.** They do not know what a realm, a client or a credential
is. Prefer plain wording over the table above wherever the English already avoids jargon. Where the
English *does* leak jargon — "device code charset", "WebAuthn is not supported", "Invalid OTP code" —
translate it faithfully and flag it: the fix is a copy change in English first.

**`account.yaml` — end users too.** The signed-in user's self-service area is routed inside the admin
shell but read by the account holder, not by an operator. Same plain register as `auth.yaml`.

**Every other catalogue — administrators.** The table above applies directly. An administrator
configuring authentication needs precision more than approachability.

## Second person

**Drop the pronoun wherever Chinese allows it.** `会话已过期，请重新登录。` rather than
`您的会话已过期`. Use **您** only when possession must be explicit, and never 你.

This is the most visible single choice in the product and nothing fixed it before, so two catalogues
translated by two people would have diverged on every other sentence.

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

## One collision the glossary cannot remove

`realm` is 域, and an organization's **domain** is also 域 in ordinary Chinese. They appear together
in `client-scope` and `organization`. Render the organization's domain as **域名** to keep them apart,
and accept that this is a workaround rather than a clean distinction.

`permission` is 权限, which leaves nothing for the English "right" / "access right". Where the source
says "right", use **权利** and flag it — asserting 权限 would claim a link to the bitmask permission
model that the English does not make.

## Tokens the user has to retype are never translated

A confirmation dialog that asks an administrator to type `delete`, or a realm name, is asking for an
exact string match. Translating it to 删除 forces the user onto an IME to complete a destructive
action. Keep the ASCII token and translate the sentence around it.

## SAML wire values stay in English

`Entity ID`, `Assertion Consumer Service URL`, `Name ID`, `NameFormat`. The English copy itself says
these are values the administrator copies from the application's own SAML settings page, so they are
strings to match against a third-party UI, not prose. Translating them makes the matching harder.

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
