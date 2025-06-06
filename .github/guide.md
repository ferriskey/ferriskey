## Test CI with ACT

```
act -W '.github/workflows/release-doc.yml' \
  -s GITHUB_TOKEN="$(gh auth token)" \
  -s SONAR_TOKEN="TOKEN" \
  -s SONAR_HOST_URL="http://sonarqube.ferriskey.bonnal.cloud"
```
