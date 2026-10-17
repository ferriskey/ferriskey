# CNCF Sandbox application

Draft of the answers to the [CNCF Sandbox application form](https://github.com/cncf/sandbox/blob/main/.github/ISSUE_TEMPLATE/application.yml). The application is filed as an issue in [cncf/sandbox](https://github.com/cncf/sandbox); this file is the reviewed source the issue is pasted from. Preparation is tracked in [#1623](https://github.com/ferriskey/ferriskey/issues/1623).

Sections follow the form in order. Facts are as of 2026-10-17 and are refreshed on the day of submission. Items marked [to confirm] are open with the maintainers.

## Basic project information

### Project summary

FerrisKey is an open source, cloud native Identity and Access Management system written in Rust: OpenID Connect, OAuth 2.0, SAML 2.0, MFA and multi-tenant realms, packaged as Helm charts and a Kubernetes operator.

### Project description

FerrisKey is an open source Identity and Access Management system written in Rust. It authenticates users and services for any application over OpenID Connect, OAuth 2.0 and SAML 2.0, with multi-factor authentication (TOTP, WebAuthn passkeys, magic links, recovery codes), identity brokering to upstream providers, organizations for B2B tenancy, security event auditing and webhooks. Every user, client, role, credential and setting belongs to a realm, so one deployment isolates many tenants. It ships as multi-architecture container images, two Helm charts and a Kubernetes operator with a `FerrisKeyCluster` custom resource. PostgreSQL is the only required dependency.

Why it is needed: self-hosted identity today means either a large Java platform whose operational weight (JVM memory, startup time, clustering, caches) is out of proportion for most teams, or vendor-led projects whose hosted tiers and licensing shape the roadmap. FerrisKey is a complete IAM that a platform team can read, deploy and operate on any Kubernetes cluster: stateless API and console replicas with all state in PostgreSQL, a single API process measured between 37 and 69 MiB resident while creating 1,000 realms, secure defaults (PKCE required for new clients, outbound calls to private networks refused, Argon2 password hashing), and no outbound call needed to operate. Users can be imported from Keycloak, Zitadel or Supabase with their password hashes, so adopting FerrisKey does not require a password reset campaign.

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

The public roadmap has three columns: what FerrisKey provides today, what is being built next, and longer-term explorations. [ROADMAP.md](https://github.com/ferriskey/ferriskey/blob/main/ROADMAP.md) is the entry point from the repository: every started item is an issue labelled `epic` and attached to a milestone. At the time of writing the v0.10 milestone holds server-side pagination on every listing (#1531), GDPR compliance and user self-service (#992) and a session-aware account chooser (#671); pooled realms for hosting many tenants on one instance (#1610) and the OAuth 2.0 Device Authorization Grant (#1020) follow. The maintainers review the roadmap at each minor release; items are proposed through GitHub Discussions under the RFC process in [GOVERNANCE.md](https://github.com/ferriskey/ferriskey/blob/main/GOVERNANCE.md), with lazy consensus (72 hours) and a maintainer vote when needed. The project is pre-1.0 and releases a minor version roughly monthly (0.7.0 on 2026-06-30, 0.8.0 on 2026-09-15, 0.9.0 on 2026-10-03).

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

FerrisKey is not the upstream of any product. No company owns the code, the trademark, the domain or the accounts; the assets are held by the founder personally and will be transferred to the Linux Foundation under the Contribution Agreement.

The relationship with companies is the following. Cloud IAM, a managed Keycloak hosting company, is the first and largest sponsor: it funds the time the founder, Nathaël Bonnal, spends on the project, and its CTO, Luis Rubiera, is one of the five maintainers. Cloud IAM may offer a managed FerrisKey service in the future; that service would run the same public releases as everyone else, and only its internal infrastructure and control plane tooling would stay outside the project. Sponsorship buys visibility in the README and nothing else: no say in the roadmap, no priority in reviews, no seat among the maintainers. France Nuage, a French open source cloud provider, sponsors the project and runs it in production. Hardis Group employs one maintainer and has no product relationship with the project. The three other maintainers are independent.

These commitments are written in the [Sponsors and open source commitments](https://github.com/ferriskey/ferriskey/blob/main/GOVERNANCE.md#sponsors-and-open-source-commitments) section of GOVERNANCE.md and were first stated publicly in [#1186](https://github.com/ferriskey/ferriskey/issues/1186): Apache-2.0 only, no CLA, no dual licensing, no open-core model, no enterprise edition, no feature held back for a paid product, and maintainers acting as individuals with affiliations kept current in MAINTAINERS.md. Joining the CNCF is the way the project anchors this independence in a neutral foundation.

### Dependency on paid cloud, community, and SaaS services or subscriptions

The project runs on free tiers and a few small paid items, all paid personally by the founder [to confirm]:

- the `ferriskey.rs` domain (registrar fee, yearly) [to confirm amount]
- hosting of ferriskey.rs and docs.ferriskey.rs on an OVH server [to confirm plan and monthly cost]
- Proton Mail for the `@ferriskey.rs` addresses (security@, conduct@) [to confirm plan and monthly cost]

Free of charge: GitHub (public organization, Actions, Packages for the container images and Helm charts on ghcr.io, Discussions, Sponsors), Discord, OpenSSF Scorecard and Best Practices. No cloud credits are consumed; CI runs on GitHub-hosted runners. Estimated total: under 50 USD per month [to confirm]. None of these services affects how the project is developed or used; an adopter needs nothing but the container images or the Helm charts.

## Cloud native context

### Why CNCF?

Three reasons, in the order the maintainers rank them.

1. Neutral ownership. The project promised its users in [#1186](https://github.com/ferriskey/ferriskey/issues/1186) and in GOVERNANCE.md that no company would own it. Today the name, the domain and the accounts are held by one person. Transferring them to the Linux Foundation, with CNCF governance expectations on top, is the only way to make that promise structural rather than personal. Adopters in regulated and sovereignty-sensitive sectors ask for exactly that before depending on an identity provider.
2. Security review. An IAM is a security product. The project has an OpenSSF Best Practices passing badge, a Scorecard, a threat model and a security policy with response targets, and it handled a critical advisory (GHSA-v2hq-9v44-43ff, fixed in 0.8.0) by building a cross-realm regression suite. What it lacks is outside review: the maintainers want the joint security self-assessment with TAG Security and Compliance, the General Technical Review process, and the fuzzing and audit programs the CNCF opens to its projects.
3. Community reach. The maintainers and most adopters are in France. The CNCF is where the platform engineers who would deploy FerrisKey already are, and where identity projects (Keycloak, Dex, SPIFFE and SPIRE, OpenFGA, cert-manager) already coordinate. Sandbox membership is the way to recruit maintainers and adopters outside the founding circle, which the project knows is its main weakness today.

### Benefit to the landscape

The landscape has one complete self-hosted identity provider, Keycloak, and specialized pieces around it. FerrisKey adds a second complete option with a different set of trade-offs: a Rust service with a small footprint, stateless replicas with all state in PostgreSQL, secure defaults enforced in code, a Kubernetes operator with a custom resource, and no SPI or plugin model that fragments deployments. It gives the landscape:

- a migration path: `ferris-ctl realm import` moves realms and users with their password hashes from Keycloak, Zitadel and Supabase, so teams can change identity provider without a reset campaign;
- a standards-only integration surface: any OpenID Connect, OAuth 2.0 or SAML 2.0 library works, and the console itself is a public OpenID Connect client, so the project exercises its own protocol endpoints;
- an IAM designed for multi-tenancy from the start, with realm isolation tested in CI against cross-realm access, and pooled realms for providers hosting many tenants on the roadmap;
- a reference for secure defaults in the identity space: PKCE required for new clients, outbound destinations on private networks refused, distroless non-root images with a read-only root filesystem, a per-realm PII mode on audit events.

### Cloud native 'fit'

FerrisKey is built to run on Kubernetes as a set of stateless, horizontally scalable services with an external PostgreSQL:

- Packaging: multi-architecture images on ghcr.io with semver and sha tags, two Helm charts published as OCI artifacts, a Kubernetes operator reconciling a `FerrisKeyCluster` custom resource (`ferriskey.rs/v1alpha1`) into Deployments, Services, Secrets and a migration Job.
- Runtime: API and console are stateless; sessions and tokens live in PostgreSQL, so replicas scale without sticky sessions and roll without downtime. Images are distroless, non-root, read-only root filesystem, all capabilities dropped.
- Observability: Prometheus metrics on `/metrics` with a `ServiceMonitor` option, OpenTelemetry tracing over OTLP, structured JSON logs, liveness and readiness probes.
- Exposure and secrets: Ingress or Gateway API `HTTPRoute` rendered by the chart, cert-manager annotations in the documented example, admin and database credentials from Secrets or an external secret manager, an Argo CD installation guide.
- Supply chain: pinned toolchains, `cargo-deny` on licenses and advisories in CI, Dependabot, OpenSSF Scorecard and Best Practices badge, container image signing with cosign in progress ([#1728](https://github.com/ferriskey/ferriskey/pull/1728)).

The Day 0 General Technical Review in [docs/cncf/general-technical-review.md](https://github.com/ferriskey/ferriskey/blob/main/docs/cncf/general-technical-review.md) details the design, installation and security answers.

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

- Keycloak (CNCF incubating): the largest overlap. Both are complete identity providers with realms, OpenID Connect, OAuth 2.0, SAML 2.0, MFA, brokering and an admin console. FerrisKey differs in implementation language and footprint (Rust, tens of MiB per process, no JVM, no distributed cache), in architecture (hexagonal domain tested at the HTTP boundary, no SPI), in its secure-by-default posture, and in being younger with a smaller feature surface (no LDAP or Kerberos federation, no authorization services module). FerrisKey ships a migration from Keycloak, so the two coexist in the landscape the way several ingress controllers or several databases do.
- Dex (CNCF sandbox): partial overlap on the OpenID Connect issuer and federation. Dex is a connector-based identity broker without a user store, MFA, SAML identity provider or console; FerrisKey covers those.
- OpenFGA (CNCF): no functional overlap. FerrisKey authorizes its own admin API with a Cedar-based policy crate; it does not aim to be a general-purpose authorization service.

### Similar projects

In the CNCF: Keycloak and Dex, discussed above.

Elsewhere, self-hosted identity providers with a comparable scope: Zitadel, Ory (Kratos and Hydra), Authentik, Authelia, Casdoor, Logto, SuperTokens, Hanko. Written in Rust: Kanidm (identity and LDAP server; one adopter replaced it with FerrisKey) and Rauthy (single-realm OpenID Connect provider). FerrisKey is the only one of these that combines Rust, multi-tenant realms, SAML 2.0 as identity provider, a Kubernetes operator and imports from other providers under an Apache-2.0-only license with no hosted or enterprise tier.

### Landscape

A pull request adding FerrisKey under Provisioning / Security and Compliance is open: [cncf/landscape#5206](https://github.com/cncf/landscape/pull/5206) (validation passing, awaiting merge). [to confirm on submission day: merged or still open]

### Insights

No. FerrisKey is not yet listed on LFX Insights.

## CNCF policies

### Trademark and accounts

- [x] If the project is accepted, I agree to donate all project trademarks and accounts to the CNCF

### IP policy

- [x] If the project is accepted, I agree the project will follow the CNCF IP Policy

### Will the project require a license exception?

N/A. The project uses the Apache 2.0 license. All contributions are made under the Developer Certificate of Origin, and no CLA is used.

### Does the project utilize any dependencies with licenses not on the allowlist or an approved exception?

Licenses are enforced by `cargo-deny` in CI with the CNCF allowlist as the allowed set ([deny.toml](https://github.com/ferriskey/ferriskey/blob/main/deny.toml)). The resolved dependency graph of the Rust workspace (607 third-party crates, normal dependencies only) and of the console (244 production packages) was checked on 2026-10-17 against the allowlist and the exception list. Four groups need attention:

1. `webauthn-rs`, `webauthn-rs-core`, `webauthn-rs-proto`, `webauthn-attestation-ca`, `base64urlsafedata` (0.5.5), MPL-2.0. Unmodified crates fetched from crates.io at build time; they implement the WebAuthn relying party for passkeys. No exception exists yet; the project will file a license exception request under the standard "unmodified, retrieved at build time" scope used for the existing MPL-2.0 exceptions.
2. The ICU4X crates (`icu_collections`, `icu_locale_core`, `icu_normalizer`, `icu_normalizer_data`, `icu_properties`, `icu_properties_data`, `icu_provider`, `litemap`, `potential_utf`, `tinystr`, `writeable`, `yoke`, `yoke-derive`, `zerofrom`, `zerofrom-derive`, `zerotrie`, `zerovec`, `zerovec-derive`), Unicode-3.0. Pulled transitively through the `url` and `idna` crates for internationalized domain names, unmodified from crates.io. No exception exists yet; the project will file a request, on the model of the blanket exception for Unicode-DFS-2016 ([cncf/foundation#1242](https://github.com/cncf/foundation/issues/1242)).
3. `webpki-roots` (0.26.11 and 1.0.4), CDLA-Permissive-2.0 (Mozilla CA certificate data). A blanket exception for `webpki-roots` exists for all CNCF projects ([cncf/foundation#172](https://github.com/cncf/foundation/issues/172)); it was recorded under the crate's earlier MPL-2.0 license, and the project will ask the Legal Committee to confirm it covers the current one.
4. `@fontsource-variable/geist` in the console, OFL-1.1. Font files only, no code. A precedent exists for OFL-1.1 fonts (font-awesome, all CNCF projects); the project will either file a request or replace the font before onboarding.

Everything else is Apache-2.0, MIT, BSD, ISC, Zlib or 0BSD, or a combination of those. The build-only dependency `ar_archive_writer` (Apache-2.0 WITH LLVM-exception) is not shipped in any artifact.

### Project "Domain Technical Review"

The Day 0 General Technical Review is answered in [docs/cncf/general-technical-review.md](https://github.com/ferriskey/ferriskey/blob/main/docs/cncf/general-technical-review.md) (template v1.0.1, updated at each minor release), together with a threat model in [docs/cncf/threat-model.md](https://github.com/ferriskey/ferriskey/blob/main/docs/cncf/threat-model.md). The project has not yet presented to TAG Security and Compliance; the maintainers will request a slot before the TOC session. [to confirm: replace the links with the v0.10 tag and add the TAG meeting notes if the presentation happens before submission]

## Contact information

### Application contact email(s)

luis.daniel.rubiera.guzman@gmail.com, pro.nathaelbonnal@gmail.com, baptiste.parmantier.pro@gmail.com

### Contributing or sponsoring entity signatory information

Or, if an individual or individual(s):

| Name | Country | Email address |
|-----------|-----------|-----------|
| Nathaël Bonnal | France | pro.nathaelbonnal@gmail.com |
| Baptiste Parmantier | France | baptiste.parmantier.pro@gmail.com |

No company or legal entity owns project assets. The ferriskey.rs domain, the GitHub organization, the Discord server and the GitHub Sponsors account are held by Nathaël Bonnal personally and will be transferred under the [Contribution Agreement](https://github.com/cncf/foundation/blob/main/agreements/Sample%20Contribution%20Agreement%20(2025).pdf). No trademark has been filed for the FerrisKey name or logo in any jurisdiction; both are assigned as unregistered marks.

## Additional information

### CNCF contacts

None yet. No member of the TOC or of a TAG has reviewed the project. [to confirm: add any contact made through TAG Security and Compliance before submission]

### Additional information

- Age and activity: first commit on 2025-03-19; 961 commits on `main`, 52 contributors in total and 19 in the last six months as of 2026-10-17; 725 stars and 98 forks. Releases are tagged with release candidates first; the last three minor releases were 0.7.0 (2026-06-30), 0.8.0 (2026-09-15) and 0.9.0 (2026-10-03).
- Maintainers: five, from four affiliations (three independent, Cloud IAM, Hardis Group), listed with affiliations in MAINTAINERS.md; the maintainer ladder and the lazy-consensus process are in GOVERNANCE.md.
- Adopters: France Nuage (service provider, production), Natalia (production), Gilded Health (replaced Kanidm), Nudibranches Technologies (consultancy). Sponsors: Cloud IAM, France Nuage, Gilded Health, Nudibranches.
- Security: OpenSSF Best Practices passing badge ([bestpractices.dev/projects/11019](https://www.bestpractices.dev/projects/11019)), OpenSSF Scorecard 6.6 on 2026-10-17, SECURITY.md with response targets, a published critical advisory (GHSA-v2hq-9v44-43ff, fixed in 0.8.0, CVE requested) followed by a cross-realm regression suite, a threat model, distroless non-root images, `cargo-deny` on advisories and licenses, image signing in progress.
- Known gaps the maintainers are working on: the maintainer base is concentrated in France and partly in one sponsor's orbit; the chart has no PodDisruptionBudget template yet; no external security audit has been done; no TAG presentation yet.
- The maintainers understand that the TOC vote and the signed Contribution Agreement both precede any claim of being a CNCF project.
