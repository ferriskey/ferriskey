# Security Policy

FerrisKey is an Identity and Access Management system. A vulnerability in it can expose every application that relies on it, so reports are handled as a priority.

## Reporting a vulnerability

Do not open a public issue, pull request or Discord message for a vulnerability.

Report it through one of these private channels:

- [Report a vulnerability](https://github.com/ferriskey/ferriskey/security/advisories/new) in the Security tab of this repository (preferred).
- Email <security@ferriskey.rs>.

Include what you can of the following: a description of the problem, the affected versions, steps or a proof of concept that reproduce it, the impact you see, and any mitigation you know of.

## What happens next

| Step | Target |
| --- | --- |
| Acknowledgement of the report | 3 business days |
| Triage and severity assessment, shared with the reporter | 7 days |
| Fix released for a critical or high severity issue | 30 days |
| Public disclosure | 90 days after the report at the latest |

The maintainers listed in [MAINTAINERS.md](./MAINTAINERS.md) handle the report. They may ask you for more details and will keep you informed of the progress.

## Embargo and disclosure

Reports stay private until a fix is released. Disclosure is coordinated with the reporter and happens when the fix is available, or 90 days after the report, whichever comes first. If a vulnerability is already public or actively exploited, the fix and the advisory are published as soon as possible.

Advisories are published on the repository [Security Advisories](https://github.com/ferriskey/ferriskey/security/advisories) page and get a CVE through GitHub. Reporters are credited in the advisory unless they ask not to be.

## Supported versions

FerrisKey is pre-1.0 and releases minor versions regularly. Security fixes are released in the latest minor version only. Upgrade to the latest release to receive them.

| Version | Supported |
| --- | --- |
| latest minor release (currently 0.9.x) | yes |
| older releases | no |

## Scope

This policy covers the code in this repository: the API, the core domain, the web console, the operator, the Helm charts and the published container images. For a vulnerability in a dependency, report it to the dependency's maintainers first and tell us if FerrisKey is affected.
