# GDPR and ISO 27001 Deployment Guide

This guide helps operators deploy FerrisKey in a way that supports compliance with the
General Data Protection Regulation (GDPR) and with information security management
frameworks such as ISO 27001.

**Scope.** This document covers the deployment and operational aspects of FerrisKey. It
does not constitute legal advice. Operators should review their obligations with their
data protection officer (DPO) and legal counsel.

---

## 1. Editor vs. Deployer Responsibility Matrix

FerrisKey is open-source software published by the FerrisKey project maintainers
(the "editor"). The entity that deploys and operates FerrisKey in a production
environment (the "deployer", also called the data controller or data processor under
GDPR depending on context) bears full responsibility for the choices it makes at the
infrastructure level.

The table below allocates responsibility across the most common compliance domains.

| Domain | FerrisKey (editor) | Deployer (operator) |
|--------|-------------------|---------------------|
| **Hosting infrastructure** | Publishes container images and Helm charts; no opinion on hosting location | Chooses hosting provider, geographic region, and data residency |
| **Data residency** | Multi-tenant realm isolation in-product; no data leaves the operator's database | Selects a cloud region or on-premises location that satisfies applicable transfer restrictions (e.g., GDPR Chapter V adequacy decisions or SCCs) |
| **Backups** | No built-in backup tooling | Implements and tests PostgreSQL backup and restore procedures; encrypts backup files at rest |
| **Key management** | Generates RSA-2048 signing key pairs per realm; stores them in the `jwt_keys` table | Encrypts the database at rest so private keys are protected; optionally externalises key storage to a dedicated secrets manager |
| **Data retention** | Provides configurable token lifetimes and audit logs; no automatic purge of user or audit data | Defines and enforces data retention schedules; deletes or anonymises personal data when the retention period ends |
| **Breach notification** | Maintains a responsible disclosure policy and issues security advisories | Detects incidents via monitoring (application logs, SeaWatch audit module); notifies the supervisory authority within 72 hours (GDPR Art. 33) and affected data subjects without undue delay (Art. 34) |
| **Data Processing Agreement (DPA)** | Not a data processor in the GDPR sense (publishes software only) | Signs DPAs with sub-processors (cloud providers, CDN, SaaS monitoring tools, etc.); maintains records of processing activities (Art. 30) |
| **DPIA** | Does not conduct DPIAs on behalf of deployers | Conducts a Data Protection Impact Assessment when processing is likely to result in high risk (GDPR Art. 35), for example in healthcare or high-volume identity processing |
| **Sub-processor management** | Not applicable (no SaaS offering) | Maintains an up-to-date list of sub-processors; ensures each has adequate data protection measures in place |
| **Security hardening** | Publishes hardened container images (Wolfi/Alpine-based, 0 known CVEs at release); see `docs/docker-hardening-report.md` | Applies OS-level hardening to host nodes; restricts network access; manages Kubernetes RBAC and pod security |
| **TLS termination** | Does not handle TLS internally; exposes an HTTP port | Terminates TLS ≥ 1.2 (recommended: TLS 1.3) at a load balancer or ingress controller in front of FerrisKey |
| **Monitoring and alerting** | Exposes a Prometheus `/metrics` endpoint and structured JSON logs; provides a SeaWatch audit API | Scrapes metrics; sets up alerting; retains logs for an appropriate period; restricts log access |
| **User data deletion (right to erasure)** | Provides user management APIs (see data subject rights work in #992) | Implements operational processes to action erasure and portability requests within GDPR deadlines |
| **DPO appointment** | Not applicable | Appoints a DPO where required (GDPR Art. 37); documents the appointment |

---

## 2. Secret and Password Handling

### 2.1 User Credential Hashing

FerrisKey hashes all user passwords before storage. The implementation uses
**Argon2id** (version 0x13) with the following parameters:

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Algorithm variant | Argon2id | Resistant to both side-channel and GPU brute-force attacks |
| Memory cost | 7 168 KiB | Aligned with Keycloak's production profile |
| Iteration count | 5 | Provides adequate time-hardening at the above memory cost |
| Parallelism | 1 | Single-threaded; avoids resource contention |
| Output length | 32 bytes | Standard digest size |
| Salt | Random, per-password, generated via `OsRng` | Prevents rainbow-table attacks |

Password hashes are stored in the `credentials` table. The raw password is never
persisted. Verification happens via constant-time comparison inside the Argon2
crate.

Magic-link tokens are hashed with the same Argon2id routine before storage in the
`email_verification_tokens` and `password_reset_tokens` tables.

### 2.2 JWT Signing Keys

FerrisKey generates an RSA-2048 key pair for each realm on first use. The key pair is
stored in the `jwt_keys` table as PEM-encoded strings. JWTs are signed with RS256.

The public key is exposed via the standard OIDC JWKS endpoint (`/realms/{realm}/.well-known/openid-configuration`) so that resource servers can verify tokens without
trusting FerrisKey's database.

**Operator responsibilities for key material:**

- **Encrypt the database at rest** so that PEM private keys stored in `jwt_keys` are
  not readable from raw disk or backup files. See Section 6 below.
- **Key rotation**: FerrisKey does not currently perform automatic key rotation.
  Operators who require periodic rotation must generate a new key pair (by clearing the
  `jwt_keys` row for a realm and restarting) and update any out-of-band systems that
  cache the old public key. In-flight access tokens signed with the previous key will
  become unverifiable after rotation; plan a rotation window accordingly.
- **Secrets manager integration**: Operators with stricter key management requirements
  (e.g., FIPS 140-2 HSM) should store the private key in a dedicated secrets manager
  (HashiCorp Vault, AWS Secrets Manager, Azure Key Vault) and adapt the keystore
  repository implementation accordingly.

### 2.3 Application Secrets and Environment Variables

The following secrets must be set via environment variables or a secrets manager and
must never be committed to source control:

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | PostgreSQL connection string including password |
| `ADMIN_PASSWORD` | Password for the initial admin account |
| Any SMTP credentials | Used by the mail module for email verification and magic links |

Kubernetes operators should store these in `Secret` objects (not `ConfigMap`) and
restrict access with RBAC. When using the FerrisKey Helm chart, pass secrets via
`--set-string` or a sealed-secrets mechanism rather than plain `values.yaml` files.

### 2.4 TLS in Transit

FerrisKey listens on plain HTTP (`PORT`, default 3333). All traffic in transit must
be encrypted by the operator's infrastructure:

- Deploy a TLS-terminating load balancer or ingress controller (e.g., AWS ALB,
  Google Cloud Load Balancing, nginx Ingress, Traefik) in front of FerrisKey.
- Use TLS 1.2 as a minimum; TLS 1.3 is strongly recommended.
- Include FerrisKey's domain in your certificate; automate renewal (e.g., cert-manager
  with Let's Encrypt or your CA).
- Internal pod-to-pod traffic (FerrisKey to PostgreSQL) should also be encrypted;
  configure PostgreSQL's `sslmode=require` or `verify-full` in `DATABASE_URL`.

### 2.5 Recovery Codes and MFA Secrets

TOTP seeds and WebAuthn credential blobs are stored in the `trident` module tables.
These are personal data and cryptographic material simultaneously. Apply the same
at-rest encryption to the database that covers `jwt_keys`.

Recovery codes are hashed with Argon2id before storage (same routine as passwords);
the plain codes are shown to the user once and never retained.

---

## 3. Privacy-by-Design and Privacy-by-Default (GDPR Art. 25)

Article 25 requires that privacy be embedded into system design and that the most
privacy-preserving defaults be applied by default.

FerrisKey's design decisions that support Art. 25 compliance:

| Principle | FerrisKey implementation |
|-----------|--------------------------|
| **Data minimisation** | JWT claims include only the scopes requested by the client application. Identity fields (`preferred_username`, `email`) are omitted from tokens when the corresponding scope is not present. |
| **Purpose limitation** | Each realm is an isolated processing environment. Data from one realm is never accessible to services authenticated against another. |
| **Access control by default** | Every API endpoint requires a valid bearer token. Admin operations require a role check enforced by the `FerriskeyPolicy` layer. |
| **Short-lived tokens by default** | Access tokens expire after 5 minutes; ID tokens expire after 5 minutes. Refresh tokens expire after 24 hours. These values are configurable per realm to be shorter, never longer by default. |
| **Least-privilege scopes** | Clients request only the scopes they need. The scope list is validated against realm configuration on every token request. |
| **Audit trail** | The SeaWatch module records security events (login, logout, token issuance, MFA registration, etc.) per realm, providing a tamper-evident trail (#1097). |
| **PII pseudonymisation in audit logs** | Planned in #1098: subject identifiers in audit records will be pseudonymised so that individual lookup is possible for authorised operators but not trivially readable in log exports. |
| **Configurable retention** | Token lifetimes are configurable. Operators define deletion schedules for user accounts and audit logs to match their retention policy. |

**Default settings an operator should verify before going to production:**

1. Change the admin password from the development default (`admin`).
2. Set `ALLOWED_ORIGINS` to the exact list of your frontend domain(s) rather than a
   wildcard.
3. Enable the password policy for each realm (minimum length ≥ 12, require uppercase,
   lowercase, digits, and special characters).
4. Enable account lockout (see #1095) to limit brute-force attempts.
5. Require MFA for privileged users using the required-action mechanism (`configure_otp`
   or `configure_passkey`).

---

## 4. Compliance Features Reference (EPIC #1090)

The following in-product features support GDPR and ISO 27001 technical controls.

| Issue | Feature | ISO 27001 control reference | GDPR reference |
|-------|---------|-----------------------------|----|
| #1091 | PKCE on the authorization code flow (RFC 7636) | A.9.4 — System and application access control | Art. 25 (security by design), Art. 32 (security of processing) |
| #1092 | Refresh token rotation with reuse detection | A.9.4 — Access control; A.12.4 — Logging | Art. 32 |
| #1093 | Active session management API (list and revoke sessions) | A.9.4 — Session management | Art. 17 (right to erasure), Art. 32 |
| #1094 | Configurable password policy per realm (length, complexity, max age) | A.9.4.3 — Password management system | Art. 32 |
| #1095 | Account lockout and anti-brute-force protection | A.9.4.5 — Secure log-on procedures | Art. 32 |
| #1096 | MFA enforcement policy (require TOTP or passkey per realm) | A.9.4 — MFA; A.6.3 — Information security awareness | Art. 32 |
| #1097 | Tamper-evident audit log with hash chaining | A.12.4 — Logging and monitoring; A.18.1 — Compliance | Art. 5(2) (accountability) |
| #1098 | Pseudonymisation of PII in audit log exports | A.8.2 — Information classification | Art. 4(5), Art. 25, Art. 89 |
| #1099 | Encryption at rest for database fields containing secrets | A.10.1 — Cryptographic controls | Art. 32(1)(a) |
| #992  | Data subject rights API (access, portability, erasure) | A.18.1 — Compliance with legal requirements | Art. 15–20 |

> **Note on ISO 27001 certification:** ISO 27001 certifies an organisation's
> Information Security Management System (ISMS), not a software product. Running
> FerrisKey does not make an organisation ISO 27001 certified. The controls above
> provide evidence that can contribute to an ISMS certification audit, but the operator
> must demonstrate overall governance, risk management, and continuous improvement
> across its full ISMS scope.

---

## 5. Data Processing Agreement Template

Operators who act as data processors on behalf of their customers must enter into a DPA
with those customers (GDPR Art. 28). The template below provides a starting point.
Review and adapt it with your legal counsel before use.

---

### Data Processing Agreement (Template)

**Between:**

**Controller:** `[Controller legal name]`, `[registered address]`, `[country]`
("Controller")

**Processor:** `[Operator/Deployer legal name]`, `[registered address]`, `[country]`
("Processor")

Together "the parties".

---

#### 1. Subject Matter and Duration

1.1 The Processor processes personal data on behalf of the Controller for the purpose
of providing identity and access management services using FerrisKey software, as
described in the main service agreement ("Main Agreement") between the parties.

1.2 This DPA is effective from the date of the Main Agreement and remains in force
until it is terminated in accordance with clause 9.

---

#### 2. Nature and Purpose of Processing

2.1 The Processor operates FerrisKey as an identity provider. Processing includes:

- Authentication: verification of user credentials.
- Authorisation: issuance of access tokens scoped to the Controller's applications.
- Audit logging: recording of security events per realm.
- Account management: creation, modification, and deletion of user accounts and roles.

2.2 The Processor processes personal data only as instructed by the Controller and does
not use the data for its own purposes.

---

#### 3. Categories of Data Subjects and Data

3.1 Data subjects may include:

- End users of the Controller's applications.
- Administrators and operators of the Controller.

3.2 Categories of personal data may include:

- Identifiers: email address, username, unique user ID.
- Authentication data: hashed password (Argon2id), MFA credentials (TOTP seed,
  WebAuthn public key), recovery code hashes.
- Audit data: IP address, timestamp, event type, user ID, realm ID.
- Profile claims: as configured by the Controller via protocol mappers.

3.3 No special categories of data (GDPR Art. 9) are processed by FerrisKey by
default. The Controller is responsible for ensuring that no special-category data is
stored as user attributes unless a lawful basis applies.

---

#### 4. Obligations of the Processor

4.1 The Processor shall:

(a) process personal data only on documented instructions from the Controller;

(b) ensure that authorised personnel are bound by confidentiality obligations;

(c) implement technical and organisational measures as described in Section 2 of the
FerrisKey GDPR and ISO 27001 Deployment Guide, including Argon2id password hashing,
RSA-2048 signed JWTs, TLS in transit, and encrypted storage at rest;

(d) assist the Controller in responding to data subject rights requests (access,
rectification, erasure, portability, restriction, objection) using the user management
APIs provided by FerrisKey;

(e) assist the Controller in meeting its obligations under Arts. 32–36 GDPR
(security, breach notification, DPIA, prior consultation);

(f) at the Controller's choice, delete or return all personal data at the end of
service provision;

(g) make available all information necessary to demonstrate compliance with Art. 28
GDPR and allow for audits by the Controller or an auditor mandated by the Controller.

---

#### 5. Sub-processors

5.1 The Processor shall not engage sub-processors without prior written authorisation
from the Controller, either specific or general.

5.2 Where the Processor uses sub-processors (e.g., cloud infrastructure provider,
managed PostgreSQL service), it shall impose data protection obligations on them
equivalent to those in this DPA.

5.3 Current sub-processors: `[list, including name, location, purpose]`.

---

#### 6. International Transfers

6.1 The Processor shall not transfer personal data outside the EEA without ensuring
that an appropriate transfer mechanism under GDPR Chapter V applies (adequacy decision,
Standard Contractual Clauses, Binding Corporate Rules, or other approved instrument).

---

#### 7. Security Incident Notification

7.1 The Processor shall notify the Controller without undue delay, and in any event
within **24 hours**, after becoming aware of a personal data breach. Notification
shall include, to the extent available:

- a description of the nature of the breach;
- the categories and approximate number of data subjects concerned;
- the categories and approximate number of personal data records concerned;
- the likely consequences of the breach;
- measures taken or proposed to address the breach.

7.2 The Controller is responsible for notifying the relevant supervisory authority
within 72 hours of becoming aware of the breach (GDPR Art. 33).

---

#### 8. Confidentiality and Return/Deletion of Data

8.1 On termination, the Processor shall, at the Controller's written request and
within 30 days, either delete all personal data or provide an export in a portable
format and then delete all copies, unless applicable law requires retention.

---

#### 9. Duration and Termination

9.1 This DPA terminates automatically when the Main Agreement terminates or expires.

9.2 Clauses relating to confidentiality, return/deletion of data, and audit rights
survive termination for a period of 5 years.

---

#### 10. Governing Law

This DPA shall be governed by the laws of `[jurisdiction]`.

---

**Signatures:**

| Controller | Processor |
|-----------|----------|
| Name: ________________ | Name: ________________ |
| Title: ________________ | Title: ________________ |
| Date: ________________ | Date: ________________ |

---

## 6. Operational Deployment Checklist

Use this checklist before promoting a FerrisKey deployment to production.

### Network and TLS

- [ ] TLS 1.2 or higher is terminated at the ingress/load balancer.
- [ ] TLS 1.3 is preferred and negotiated by default.
- [ ] The PostgreSQL connection uses `sslmode=require` or `verify-full`.
- [ ] Internal pod-to-pod traffic is network-isolated (Kubernetes NetworkPolicy or
      equivalent).
- [ ] `ALLOWED_ORIGINS` is set to the exact list of frontend domains; no wildcard.

### Secrets Management

- [ ] `DATABASE_URL` is stored in a Kubernetes `Secret` or secrets manager; not in
      plaintext config.
- [ ] `ADMIN_PASSWORD` is set to a strong, randomly generated value.
- [ ] SMTP credentials are stored in a `Secret`; not in plaintext config.
- [ ] The database at rest is encrypted (filesystem encryption, cloud-provider managed
      encryption, or transparent data encryption).

### Identity and Access

- [ ] The default admin password has been changed.
- [ ] A password policy has been configured for each realm (minimum length ≥ 12).
- [ ] Account lockout is configured for each realm (#1095).
- [ ] MFA is required for all administrator accounts via the `configure_otp` or
      `configure_passkey` required action (#1096).
- [ ] PKCE is enforced for all confidential clients using the authorization code flow
      (#1091).
- [ ] Refresh token rotation is enabled (#1092).

### Audit and Monitoring

- [ ] SeaWatch security event logs are forwarded to a centralised log system with
      appropriate retention (minimum 12 months recommended for ISO 27001).
- [ ] Prometheus metrics are scraped and alerting is configured for anomalous
      authentication failure rates and error rates.
- [ ] Access to the SeaWatch API is restricted to authorised admin roles.

### Backup and Recovery

- [ ] Automated PostgreSQL backups run at least daily.
- [ ] Backup files are encrypted at rest and stored in a separate location from the
      primary database.
- [ ] A recovery test has been performed to verify the restore procedure.
- [ ] Recovery Time Objective (RTO) and Recovery Point Objective (RPO) are documented
      and accepted by stakeholders.

### Data Retention

- [ ] A data retention schedule has been defined for user accounts, audit logs, and
      access tokens.
- [ ] Automated processes (cron jobs or PostgreSQL jobs) implement the retention
      schedule.
- [ ] A procedure exists to action data subject erasure requests within 30 days (GDPR
      Art. 17).

### Documentation

- [ ] A Records of Processing Activities (RoPA) document has been created (GDPR
      Art. 30).
- [ ] A DPA has been signed with each sub-processor (see Section 5 of this guide).
- [ ] A Data Protection Impact Assessment (DPIA) has been conducted if required.
- [ ] The contact details of the DPO (if applicable) are documented.

---

## 7. References

- GDPR (Regulation (EU) 2016/679): <https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679>
- ISO/IEC 27001:2022 — Information security management systems
- CNIL guidance on authentication: <https://www.cnil.fr/fr/authentification-les-recommandations-de-la-cnil>
- OWASP Password Storage Cheat Sheet: <https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html>
- RFC 7636 — PKCE: <https://datatracker.ietf.org/doc/html/rfc7636>
- FerrisKey security features (EPIC #1090): <https://github.com/ferriskey/ferriskey/issues/1090>
- FerrisKey data subject rights (#992): <https://github.com/ferriskey/ferriskey/issues/992>
- FerrisKey Docker hardening: see `docs/docker-hardening-report.md`
