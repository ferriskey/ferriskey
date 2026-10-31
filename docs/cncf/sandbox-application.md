# CNCF Sandbox application

Draft of the answers to the [CNCF Sandbox application form](https://github.com/cncf/sandbox/blob/main/.github/ISSUE_TEMPLATE/application.yml). The application is filed as an issue in [cncf/sandbox](https://github.com/cncf/sandbox); this file is the reviewed source the issue is pasted from. Preparation is tracked in [#1623](https://github.com/ferriskey/ferriskey/issues/1623).

Sections follow the form in order. Facts are as of 2026-10-31.

## Basic project information

### Project summary

FerrisKey is an open source, cloud native Identity and Access Management system written in Rust: OpenID Connect, OAuth 2.0, SAML 2.0, MFA and multi-tenant realms, packaged as Helm charts and a Kubernetes operator.

### Project description

FerrisKey is an open source Identity and Access Management system written in Rust. It authenticates users and services for any application over OpenID Connect, OAuth 2.0 and SAML 2.0, with multi-factor authentication (TOTP, WebAuthn passkeys, magic links, recovery codes), identity brokering to upstream providers, organizations for B2B tenancy, security event auditing and webhooks. Every user, client, role, credential and setting belongs to a realm, so one deployment isolates many tenants. It ships as multi-architecture container images, two Helm charts and a Kubernetes operator with a `FerrisKeyCluster` custom resource. PostgreSQL is the only required dependency.

Why it is needed: self-hosted identity today means either a large Java platform whose operational weight (JVM memory, startup time, clustering, caches) is out of proportion for most teams, or vendor-led projects whose hosted tiers and licensing shape the roadmap. FerrisKey is a complete IAM that a platform team can read, deploy and operate on any Kubernetes cluster: stateless API and console replicas with all state in PostgreSQL, a single API process that, on the same Kubernetes bench as Keycloak 26.8.1, sustains the same rate on every endpoint measured with less CPU on five of six and 618 MiB of memory at peak against 823 MiB, secure defaults (PKCE required for new clients, outbound calls to private networks refused, Argon2 password hashing), and no outbound call needed to operate. Users can be imported from Keycloak, Zitadel or Supabase with their password hashes, so adopting FerrisKey does not require a password reset campaign.

The project is Apache-2.0 only, with no CLA, no open-core tier and no enterprise edition. It is governed by five maintainers from four affiliations and runs in production at a French open source cloud provider and an AI SaaS.

### Project vs Reference Architecture/Implementation

- [x] This is a reusable open source project designed for broad adoption, not a reference architecture, reference implementation, or demonstration of patterns

## Project details

### Org repo URL

https://github.com/ferriskey

Not every repository under the organization is in scope (the organization also holds an old website, slides and demo applications). The repositories in scope are listed in the next two fields.

### Project repo URL in scope of application

https://github.com/ferriskey/ferriskey

### Additional repos in scope of the application

- https://github.com/ferriskey/ferriskey-cli (`ferris-ctl`: realm, client and user management, imports from Keycloak, Zitadel and Supabase)
- https://github.com/ferriskey/terraform-provider-ferriskey (Terraform provider generated from the OpenAPI document)
- https://github.com/ferriskey/website (ferriskey.rs and docs.ferriskey.rs)

### Website URL

https://ferriskey.rs

### Roadmap

https://ferriskey.rs/roadmap

### Roadmap context

The public roadmap has three columns: what FerrisKey provides today, what is being built next, and longer-term explorations. [ROADMAP.md](https://github.com/ferriskey/ferriskey/blob/main/ROADMAP.md) is the entry point from the repository: every started item is an issue labelled `epic` and attached to a milestone. At the time of writing the v0.10 milestone holds server-side pagination on every listing (#1531), GDPR compliance and user self-service (#992) and a session-aware account chooser (#671); pooled realms for hosting many tenants on one instance (#1610) and the OAuth 2.0 Device Authorization Grant (#1020) follow. The maintainers review the roadmap at each minor release; items are proposed through GitHub Discussions under the RFC process in [GOVERNANCE.md](https://github.com/ferriskey/ferriskey/blob/main/GOVERNANCE.md), with lazy consensus (72 hours) and a maintainer vote when needed. The project is pre-1.0 and releases a minor version roughly monthly (0.7.0 on 2026-06-30, 0.8.0 on 2026-09-15, 0.9.0 on 2026-10-03, 0.10.0 on 2026-10-27). After 0.10.1 the token hot path for machine-to-machine traffic stays a focus (`perf/token-hot-path`, v0.11 milestone): the design target is thousands of service and agent authentications per second per replica, not hundreds of human logins.

### Contributing guide

https://github.com/ferriskey/ferriskey/blob/main/CONTRIBUTING.md

### Code of Conduct (CoC)

https://github.com/ferriskey/ferriskey/blob/main/CODE_OF_CONDUCT.md

### Adopters

https://github.com/ferriskey/ferriskey/blob/main/ADOPTERS.md

### Maintainers file

https://github.com/ferriskey/ferriskey/blob/main/MAINTAINERS.md

### Security policy file

https://github.com/ferriskey/ferriskey/blob/main/SECURITY.md

### Standard or specification?

N/A. FerrisKey implements existing standards (OpenID Connect Core and Discovery, OAuth 2.0 and its extensions such as PKCE, device authorization, token exchange, introspection, revocation and dynamic client registration, SAML 2.0) and does not define a standard or specification of its own.

### Business product or service to project separation

FerrisKey is not the upstream of any product. No company owns the code, the name, the domain or the accounts; the founder holds them personally and assigns them to the Linux Foundation under the Contribution Agreement.

Cloud IAM, a managed Keycloak hosting company, is the first and largest sponsor, and its CTO, Luis Rubiera (who submits this application), is one of the five maintainers. It may offer a managed FerrisKey service in the future; that service would run the same public releases as everyone else, with only its own infrastructure and control plane tooling outside the project. Sponsorship buys visibility in the README and nothing else: no say in the roadmap, no priority in reviews, no seat among the maintainers. France Nuage sponsors the project and runs it in production; Hardis Group employs one maintainer; the three other maintainers are independent. The commitments (Apache-2.0 only, no CLA, no open-core model, no enterprise edition) are written in [GOVERNANCE.md](https://github.com/ferriskey/ferriskey/blob/main/GOVERNANCE.md#sponsors-and-open-source-commitments) and were first stated publicly in [#1186](https://github.com/ferriskey/ferriskey/issues/1186).

### Dependency on paid cloud, community, and SaaS services or subscriptions

The project runs on free tiers and a few small paid items held personally by the founder, Nathaël Bonnal, who also holds the `ferriskey.rs` domain, the GitHub organization and the Discord server:

- the `ferriskey.rs` domain (yearly registrar fee)
- hosting of ferriskey.rs and docs.ferriskey.rs on an OVH server
- Proton Mail Essentials, two seats, for the `@ferriskey.rs` addresses (security@, conduct@)

Free of charge: GitHub (public organization, Actions, Packages for the container images and Helm charts on ghcr.io, Discussions, Sponsors), Discord, OpenSSF Scorecard and Best Practices. No cloud credits are consumed; CI runs on GitHub-hosted runners. None of these services affects how the project is developed or used; an adopter needs nothing but the container images or the Helm charts.

## Cloud native context

### Why CNCF?

Three reasons, in the order the maintainers rank them.

1. Neutral ownership. The project promised its users in [#1186](https://github.com/ferriskey/ferriskey/issues/1186) and in GOVERNANCE.md that no company would own it. Today the name, the domain and the accounts are held by one person. Transferring them to the Linux Foundation, with CNCF governance expectations on top, is the only way to make that promise structural rather than personal. Adopters in regulated and sovereignty-sensitive sectors ask for exactly that before depending on an identity provider.
2. Security review. An IAM is a security product. The project has an OpenSSF Best Practices passing badge, a Scorecard, a threat model and a security policy with response targets, and it handled a critical advisory (GHSA-v2hq-9v44-43ff, fixed in 0.8.0) by building a cross-realm regression suite. What it lacks is outside review: the maintainers want the joint security self-assessment with TAG Security and Compliance, the General Technical Review process, and the fuzzing and audit programs the CNCF opens to its projects.
3. Community reach. The maintainers and most adopters are in France. The CNCF is where the platform engineers who would deploy FerrisKey already are, and where identity projects (Keycloak, Dex, SPIFFE and SPIRE, OpenFGA, cert-manager) already coordinate. Sandbox membership is the way to recruit maintainers and adopters outside the founding circle, which the project knows is its main weakness today.

### Benefit to the landscape

The landscape has one complete self-hosted identity provider, Keycloak, and specialized pieces around it. FerrisKey adds a second complete option with a different set of trade-offs: a Rust service with a small footprint, stateless replicas with all state in PostgreSQL, secure defaults enforced in code, a Kubernetes operator with a custom resource, and no SPI or plugin model that fragments deployments. It gives the landscape:

- a measured alternative rather than a claimed one: on the same cluster, database and load, FerrisKey 0.10.1 holds the same highest step as Keycloak 26.8.1 on six endpoints with less CPU on five of them and a peak memory of 618 MiB against 823 MiB, Keycloak keeping the lower p99 in places ([benchmarks page](https://docs.ferriskey.rs/en/kubernetes/benchmarks), [report](https://github.com/ferriskey/IAM-performance-tester/blob/main/reports/2026-10-31-ferriskey-vs-keycloak.md));
- a migration path: `ferris-ctl realm import` moves realms and users with their password hashes from Keycloak, Zitadel and Supabase, so teams can change identity provider without a reset campaign;
- a standards-only integration surface: any OpenID Connect, OAuth 2.0 or SAML 2.0 library works, and the console itself is a public OpenID Connect client, so the project exercises its own protocol endpoints;
- an IAM designed for multi-tenancy from the start, with realm isolation tested in CI against cross-realm access, and pooled realms for providers hosting many tenants on the roadmap;
- a reference for secure defaults in the identity space: PKCE required for new clients, outbound destinations on private networks refused, distroless non-root images with a read-only root filesystem, a per-realm PII mode on audit events.

### Cloud native 'fit'

FerrisKey runs on Kubernetes as stateless API and console replicas with all state in PostgreSQL: multi-architecture images on ghcr.io, two Helm charts published as OCI artifacts, an operator reconciling a `FerrisKeyCluster` resource, Prometheus metrics and OpenTelemetry tracing, distroless non-root images, `cargo-deny`, Dependabot and Scorecard in CI. The [General Technical Review](https://github.com/ferriskey/ferriskey/blob/main/docs/cncf/general-technical-review.md) details packaging, runtime, observability and supply chain.

The workload it is designed for is the one identity is moving to: not a few hundred human logins per second, but thousands of machine identities, services and agents authenticating, exchanging and introspecting short-lived tokens continuously. Hence the small footprint per replica, secure defaults that need no tuning, and load shedding instead of queuing: `SERVER_TOKEN_MAX_IN_FLIGHT` bounds the concurrent token requests per CPU, and the excess gets a 503 `temporarily_unavailable` with `Retry-After` (RFC 6749 section 5.2), so latency stays bounded and callers retry instead of waiting seconds for an answer. The [bench against Keycloak 26.8.1](https://docs.ferriskey.rs/en/kubernetes/benchmarks) reflects this: on `client_credentials`, `refresh`, `introspect` and `jwks` FerrisKey holds the same rate, thousands of requests per second, with about half the CPU and a fraction of the memory, while Keycloak keeps the better tail latency on human password login. Token exchange (RFC 8693) is shipped for delegation between services and agents, the roadmap lists an MCP server for agent workflows, and the token hot path keeps being optimised after 0.10.1.

### Cloud native 'integration'

- Kubernetes: Helm charts, operator and custom resource, Gateway API `HTTPRoute`, probes, `ServiceMonitor`.
- Helm: both charts are distributed through `oci://ghcr.io/ferriskey/charts`.
- Prometheus: metrics endpoint and `ServiceMonitor`; a starter dashboard is shipped.
- OpenTelemetry: traces exported over OTLP, with a ready-made SigNoz dashboard in the repository.
- cert-manager: TLS for the documented Ingress and Gateway examples.
- Argo CD: installation guide in the documentation; the migration Job runs as a hook on each deploy.
- CloudNativePG: the documented choice for a highly available PostgreSQL.
- Keycloak: FerrisKey imports realms and users, including password hashes, from Keycloak; it can also broker to Keycloak as an upstream OpenID Connect provider.
- Dex: either side can be the upstream of the other over OpenID Connect; FerrisKey adds the user store, MFA, SAML and the console that Dex leaves to others.
- OpenFGA, SPIFFE and SPIRE: complementary rather than overlapping. FerrisKey issues user and service identities over OpenID Connect; a relying party can feed them to OpenFGA for relationship-based authorization, and workloads can keep SPIFFE identities for service-to-service trust. No direct integration exists yet.

### Cloud native overlap

- Keycloak (CNCF incubating): the largest overlap. Both are complete identity providers with realms, OpenID Connect, OAuth 2.0, SAML 2.0, MFA, brokering and an admin console. FerrisKey differs in implementation language and footprint (Rust, no JVM, no distributed cache; on a shared bench against Keycloak 26.8.1 it holds the same sustained rate on six endpoints with less CPU on five of them and 618 MiB peak memory against 823 MiB, while Keycloak keeps the lower p99 on `password` and above 3,200 req/s on `introspect` and `userinfo`: [benchmarks page](https://docs.ferriskey.rs/en/kubernetes/benchmarks), [report](https://github.com/ferriskey/IAM-performance-tester/blob/main/reports/2026-10-31-ferriskey-vs-keycloak.md)), in architecture (hexagonal domain tested at the HTTP boundary, no SPI), in its secure-by-default posture, and in being younger with a smaller feature surface (no authorization services module, no configurable authentication flows, no SPI for custom code). FerrisKey ships a migration from Keycloak, so the two coexist in the landscape the way several ingress controllers or several databases do.
- Dex (CNCF sandbox): partial overlap on the OpenID Connect issuer and federation. Dex is a connector-based identity broker without a user store, MFA, SAML identity provider or console; FerrisKey covers those.
- OpenFGA (CNCF): no functional overlap. FerrisKey authorizes its own admin API with a Cedar-based policy crate; it does not aim to be a general-purpose authorization service.

### Similar projects

In the CNCF: Keycloak and Dex, discussed above.

Elsewhere, self-hosted identity providers with a comparable scope: Zitadel, Ory (Kratos and Hydra), Authentik, Authelia, Casdoor, Logto, SuperTokens, Hanko. Written in Rust: Kanidm (identity and LDAP server; one adopter replaced it with FerrisKey) and Rauthy (single-realm OpenID Connect provider). FerrisKey is the only one of these that combines Rust, multi-tenant realms, SAML 2.0 as identity provider, a Kubernetes operator and imports from other providers under an Apache-2.0-only license with no hosted or enterprise tier.

### Landscape

Yes. FerrisKey is listed under Provisioning / Security and Compliance since [cncf/landscape#5206](https://github.com/cncf/landscape/pull/5206) merged on 2026-10-19.

### Insights

No. FerrisKey is not yet listed on LFX Insights.

## CNCF policies

### Trademark and accounts

- [x] If the project is accepted, I agree to donate all project trademarks and accounts to the CNCF

### IP policy

- [x] If the project is accepted, I agree the project will follow the CNCF IP Policy

### Project naming

- [x] I have reviewed the Linux Foundation Trademark Usage Policies and confirm the project name does not use an existing trademark

### Does the project name use "kube", "k8s", or another existing project's prefix/suffix?

N/A. "FerrisKey" is a coined name after Ferris, the Rust mascot; it carries no prefix or suffix of an existing project.

### Will the project require a license exception?

N/A. The project uses the Apache 2.0 license. All contributions are made under the Developer Certificate of Origin, and no CLA is used.

### Does the project utilize any dependencies with licenses not on the allowlist or an approved exception?

`cargo-deny` enforces the CNCF allowlist in CI ([deny.toml](https://github.com/ferriskey/ferriskey/blob/main/deny.toml)). Checked on 2026-10-31 over the Rust workspace (runtime dependencies), the console's production packages and the Terraform provider, these are outside the allowlist with no approved exception yet:

- MPL-2.0: `webauthn-rs`, `webauthn-rs-core`, `webauthn-rs-proto`, `webauthn-attestation-ca`, `base64urlsafedata` 0.5.5 (the WebAuthn relying party for passkeys), and `cssparser` 0.38 with `dtoa-short` 0.3 (pulled by the `ammonia` HTML sanitizer); all unmodified, fetched from crates.io at build time.
- Unicode-3.0: the ICU4X crates (`icu_*`, `litemap`, `potential_utf`, `tinystr`, `writeable`, `yoke`, `zerofrom`, `zerotrie`, `zerovec` and their derive crates), transitively through `url` and `idna`.
- CDLA-Permissive-2.0: `webpki-roots` 0.26.11 and 1.0.10 (Mozilla CA data); the blanket exception in [cncf/foundation#172](https://github.com/cncf/foundation/issues/172) was recorded under the crate's earlier MPL-2.0 license.
- OFL-1.1: `@fontsource-variable/geist` and `@fontsource/ibm-plex-sans`, font files in the console.
- MPL-2.0, in terraform-provider-ferriskey: the HashiCorp Go modules every Terraform provider builds against (`terraform-plugin-framework`, `terraform-plugin-framework-validators`, `terraform-plugin-go`, `terraform-plugin-log`, `terraform-plugin-docs`, `terraform-plugin-testing`), unmodified, fetched at build time. Other HashiCorp MPL-2.0 modules already hold blanket exceptions for all CNCF projects.

Everything else is Apache-2.0, MIT, BSD, ISC, Zlib or 0BSD. Exception requests for these groups are filed at onboarding, or the dependency is replaced (the fonts).

### Project "Domain Technical Review"

The Day 0 General Technical Review is answered in [docs/cncf/general-technical-review.md](https://github.com/ferriskey/ferriskey/blob/main/docs/cncf/general-technical-review.md) (template v1.0.1, updated at each minor release), together with a threat model in [docs/cncf/threat-model.md](https://github.com/ferriskey/ferriskey/blob/ace8c1ac/docs/cncf/threat-model.md). The project has not yet presented to TAG Security and Compliance; the maintainers will request a slot before the TOC session.

## Contact information

### Application contact email(s)

luis.daniel.rubiera.guzman@gmail.com, pro.nathaelbonnal@gmail.com, baptiste.parmantier.pro@gmail.com

### Contributing or sponsoring entity signatory information

Or, if an individual or individual(s):

| Name | Country | Email address |
|-----------|-----------|-----------|
| Nathaël Bonnal | France | pro.nathaelbonnal@gmail.com |
| Baptiste Parmantier | France | baptiste.parmantier.pro@gmail.com |

No company or legal entity owns project assets. The ferriskey.rs domain, the GitHub organization, the Discord server and the GitHub Sponsors account are held by Nathaël Bonnal personally, and the FerrisKey name and logo are unregistered marks; all are assigned under the [Contribution Agreement](https://github.com/cncf/foundation/blob/main/agreements/Sample%20Contribution%20Agreement%20(2025).pdf).

## Additional information

### CNCF contacts

None yet. No member of the TOC or of a TAG has reviewed the project.

### Additional information

- Age and activity: first commit on 2025-03-19; 1,057 commits on `main`, 57 contributors (21 in the last six months) as of 2026-10-31; 742 stars, 103 forks. Minor releases roughly monthly, with release candidates first: 0.8.0 (2026-09-15), 0.9.0 (2026-10-03), 0.10.0 (2026-10-27), 0.10.1 (2026-10-31).
- Performance: the bench against Keycloak 26.8.1 run on 2026-10-26 found FerrisKey behind on `refresh` and `userinfo`; the fixes shipped in 0.10.1 five days later (#1863, #1870, #1875) and the 2026-10-31 run shows the gap closed ([benchmarks page](https://docs.ferriskey.rs/en/kubernetes/benchmarks), [report](https://github.com/ferriskey/IAM-performance-tester/blob/main/reports/2026-10-31-ferriskey-vs-keycloak.md)).
- Security: OpenSSF Best Practices passing badge ([bestpractices.dev/projects/11019](https://www.bestpractices.dev/projects/11019)), Scorecard 8.7 on 2026-10-31, a published critical advisory (GHSA-v2hq-9v44-43ff, fixed in 0.8.0, CVE requested) followed by a cross-realm regression suite and a threat model.
- Known gaps: the maintainer base is concentrated in France and partly in one sponsor's orbit; no external security audit yet; no TAG presentation yet.
- The maintainers understand that the TOC vote and the signed Contribution Agreement both precede any claim of being a CNCF project.
