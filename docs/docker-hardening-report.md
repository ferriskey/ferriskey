# Docker Hardening and Vulnerability Report

Date: 2026-02-13
Branch: `vk/18f7-improve-the-dock`

## Executive Summary

This update hardens the container stack and modernizes build/runtime images.

- API and webapp builds were moved to patched Wolfi-based pipelines.
- Frontend runtime was aligned to Angie official paths and startup behavior.
- Web build tooling was upgraded to Node 24 and latest pnpm at build time.
- Compose and Just workflow updates were added to support safer local lifecycle operations.

Result: Trivy shows **0 known vulnerabilities** in the new local API and webapp images, versus multiple findings in current GHCR images.

## Why These Docker Changes Were Needed

1. Outdated base/runtime packages in registry images had known vulnerabilities.
2. The Angie runtime previously served the default page when config paths targeted Nginx paths.
3. Web build toolchain needed current security patches from Node and pnpm ecosystem.
4. Local dev/test cleanup needed an explicit target to remove local images and avoid stale artifacts.

## Security Improvement (Quantified)

Trivy scan mode: `trivy image --scanners vuln`

| Image | Source | OS | Critical | High | Medium | Low | Total |
|---|---|---|---:|---:|---:|---:|---:|
| `ferriskey-local-api:hardening` | new local build | wolfi 20230201 | 0 | 0 | 0 | 0 | 0 |
| `ferriskey-local-webapp:hardening` | new local build | alpine 3.22.3 | 0 | 0 | 0 | 0 | 0 |
| `ghcr.io/ferriskey/ferriskey-api:latest` | current registry image | debian 12.13 | 2 | 4 | 30 | 61 | 97 |
| `ghcr.io/ferriskey/ferriskey-webapp:latest` | current registry image | alpine 3.21.5 | 2 | 4 | 17 | 3 | 26 |

### Net Delta

- API: `97 -> 0` vulnerabilities (**100% reduction**), including `6 -> 0` critical/high.
- Webapp: `26 -> 0` vulnerabilities (**100% reduction**), including `6 -> 0` critical/high.
- Combined: `123 -> 0` vulnerabilities across scanned images.

## Files Changed

1. `Dockerfile`
   - package refresh in build/runtime stages
   - Node 24 + latest pnpm in web build stage
   - Angie minimal runtime image
2. `docker-compose.yaml`
   - PostgreSQL image update to `postgres:18.1`
   - Postgres volume path adjusted
3. `justfile`
   - added `dev-test-rm` to remove local images with compose teardown

## Why We Changed from Nginx to Angie

1. Project runtime target is Angie, so image/config should match Angie defaults and official docs.
2. Angie official image uses:
   - config include directory: `/etc/angie/http.d/*.conf`
   - static root conventions under `/usr/share/angie/html`
3. Using Angie-native paths avoids fallback to the default welcome page and ensures intended site config is loaded.

## Why Node 24 and Latest pnpm

1. Node 24 brings newer upstream security fixes in the JS toolchain.
2. `pnpm@latest` ensures the build uses the newest pnpm patch line with security and resolver fixes.
3. Node/pnpm are used only in the **build stage**, so runtime attack surface remains small.

Note: If strict reproducibility is preferred over always-latest tooling, pinning pnpm major/minor can be considered later.

## Vulnerability Report Details

Commands executed:

```bash
trivy image --scanners vuln --format json -o reports/trivy/local-api.json ferriskey-local-api:hardening
trivy image --scanners vuln --format json -o reports/trivy/local-webapp.json ferriskey-local-webapp:hardening
trivy image --scanners vuln --format json -o reports/trivy/ghcr-api.json ghcr.io/ferriskey/ferriskey-api:latest
trivy image --scanners vuln --format json -o reports/trivy/ghcr-webapp.json ghcr.io/ferriskey/ferriskey-webapp:latest
```

### GHCR API critical/high examples

- `CVE-2025-15467` (`libssl3`) - Critical
- `CVE-2023-45853` (`zlib1g`) - Critical
- `CVE-2026-0861` (`libc-bin`, `libc6`) - High
- `CVE-2025-69419` (`libssl3`) - High
- `CVE-2025-69421` (`libssl3`) - High

### GHCR Webapp critical/high examples

- `CVE-2025-15467` (`libcrypto3`, `libssl3`) - Critical
- `CVE-2025-69419` (`libcrypto3`, `libssl3`) - High
- `CVE-2025-69421` (`libcrypto3`, `libssl3`) - High

## Maintainer Conclusion

These changes are necessary to align runtime behavior with Angie, eliminate known image-level vulnerabilities, and keep frontend build dependencies current. The resulting images reduce observed vulnerability exposure from 123 findings to 0 in this scan set.
