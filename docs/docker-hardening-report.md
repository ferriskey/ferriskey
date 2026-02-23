# Docker Hardening and Vulnerability Fix Report

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

1. There is a documented governance controversy around NGINX OSS development:
   - In February 2024, long-time core developer Maxim Dounin announced he would stop participating in nginx development run by F5 and start `freenginx`, citing management interference with security policy and loss of developer control.
2. The "life support mode" statement is only partially true and is often confused:
   - It is true for **Ingress NGINX** (the Kubernetes `ingress-nginx` project), where Kubernetes announced best-effort maintenance until March 2026, then retirement with no further releases or security fixes.
   - It is **not** an official status for the core nginx web server itself. nginx.org still shows active mainline/stable releases and recent security fixes.
3. Angie gives a practical path for this project:
   - Angie is a free nginx fork built by former nginx developers and positioned as a drop-in replacement.
   - Angie-native paths (`/etc/angie/http.d`, `/usr/share/angie/html`) match the actual runtime image behavior and prevent falling back to the default welcome page.

References:

- freenginx announcement (Maxim Dounin): <https://freenginx.org/pipermail/nginx-devel/2024-February/000000.html>
- Ingress NGINX retirement notice (Kubernetes): <https://www.kubernetes.dev/blog/2025/11/12/ingress-nginx-retirement/>
- nginx release stream (mainline/stable): <https://nginx.org/en/download.html>
- nginx change log with recent security fixes: <https://nginx.org/en/CHANGES>
- Angie docs/about (fork and release model): <https://en.angie.software/angie/docs/>

## Why Node 24

1. Node 24 brings newer upstream security fixes in the JS toolchain.
2. Node/pnpm are used only in the **build stage**, so runtime attack surface remains small.

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
