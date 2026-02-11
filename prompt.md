Extract duplicated TokenIntrospectionResponse construction into a helper.

The identical mapping from JwtClaim to TokenIntrospectionResponse is repeated for both the opaque-token path (lines 1337–1355) and the JWT-fallback path (lines 1379–1397). This violates DRY and increases the risk of the two paths diverging silently.
♻️ Proposed refactor

+    fn claims_to_introspection_response(
+        claims: JwtClaim,
+        realm_name: String,
+    ) -> TokenIntrospectionResponse {
+        TokenIntrospectionResponse {
+            active: true,
+            scope: claims.scope,
+            client_id: Some(claims.azp),
+            username: claims.preferred_username,
+            sub: Some(claims.sub.to_string()),
+            token_type: Some(match claims.typ {
+                ClaimsTyp::Bearer => "Bearer".to_string(),
+                ClaimsTyp::Refresh => "Refresh".to_string(),
+                ClaimsTyp::Temporary => "Temporary".to_string(),
+            }),
+            exp: claims.exp,
+            iat: Some(claims.iat),
+            nbf: Some(claims.iat),
+            aud: Some(claims.aud.join(" ")),
+            iss: Some(claims.iss),
+            jti: Some(claims.jti.to_string()),
+            realm: Some(realm_name),
+        }
+    }

Then use it in both paths:

-            return Ok(TokenIntrospectionResponse {
-                active: true,
-                // ... 15 lines ...
-            });
+            return Ok(Self::claims_to_introspection_response(claims, realm.name));

Also applies to: 1379-1397
🤖 Prompt for AI Agents

In `@core/src/domain/authentication/services.rs` around lines 1337 - 1356, Extract
the duplicated construction of TokenIntrospectionResponse into a single helper
function (e.g., build_token_introspection_response or
map_claims_to_introspection) that accepts the JwtClaim (claims) and the Realm
(realm) and returns a TokenIntrospectionResponse; move the field mappings
(active: true, scope, client_id from azp, username from preferred_username, sub
from sub.to_string(), token_type match on ClaimsTyp, exp, iat/nbf, aud joined,
iss, jti.to_string(), realm.name, etc.) into that helper and replace both the
opaque-token path and the JWT-fallback path returns with a call to this helper
wrapped in Ok(...).

