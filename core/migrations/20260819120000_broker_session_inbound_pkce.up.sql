-- FK-014: the broker callback built a fresh AuthSession with code_challenge NULL,
-- on the reasoning that the outbound PKCE pair (FerrisKey to the upstream IdP)
-- already covered the flow. It does not: outbound PKCE protects the upstream
-- channel, inbound PKCE protects the client-to-FerrisKey channel. A client that
-- omits session_id lands in that branch, so a code issued through brokering was
-- exchangeable with no verifier -- including for a client with require_pkce set.
--
-- These columns carry the inbound client's challenge across the broker round
-- trip so the callback can put it back on the AuthSession it creates.
ALTER TABLE broker_auth_sessions ADD COLUMN code_challenge TEXT;
ALTER TABLE broker_auth_sessions ADD COLUMN code_challenge_method TEXT;
