//! Matching of an incoming `redirect_uri` against the URIs registered for a client.
//!
//! RFC 6749 §3.1.2.2 mandates *simple string comparison* against a pre-registered
//! URI, and RFC 9700 §2.1 restates it: the redirect URI is the only thing standing
//! between an authorization code and an attacker-chosen destination, so the
//! comparison must not be clever.
//!
//! FerrisKey historically compiled every registered value as a regex and matched it
//! with `Regex::is_match`, which is a *substring search*. Two consequences, both
//! exploitable:
//!
//! - a literal `https://app.example/cb` accepted any URL merely containing it
//!   (`https://attacker.example/?next=https://app.example/cb`), and its unescaped
//!   `.` were wildcards;
//! - the seeded pattern `^/*` — "start of string, then zero or more slashes" —
//!   matched the empty string at position 0, hence every URI.
//!
//! The rule implemented here:
//!
//! 1. exact string equality accepts (this is the RFC behaviour, and covers every
//!    literal URI);
//! 2. a registered value is treated as a pattern **only** when the operator opted
//!    in by writing it anchored, `^…$`. It is then re-anchored with `\A(?:…)\z` so
//!    that an alternation cannot smuggle an unanchored branch past the outer
//!    anchors (`^a|b$` would otherwise accept `b` anywhere);
//! 3. anything else — unanchored pattern, invalid regex, or a pattern that accepts
//!    the empty string — matches nothing. Fail closed.

use regex::Regex;

/// Whether `redirect_uri` is accepted by the registered value `pattern`.
pub fn redirect_uri_matches(pattern: &str, redirect_uri: &str) -> bool {
    if pattern == redirect_uri {
        return true;
    }

    compile_anchored(pattern).is_some_and(|regex| regex.is_match(redirect_uri))
}

/// Whether any registered value accepts `redirect_uri`.
///
/// An empty set of registered values accepts nothing — callers distinguish "none
/// registered" from "none matched" before calling, since the two map to different
/// errors.
pub fn redirect_uri_matches_any<'a, I>(patterns: I, redirect_uri: &str) -> bool
where
    I: IntoIterator<Item = &'a str>,
{
    patterns
        .into_iter()
        .any(|pattern| redirect_uri_matches(pattern, redirect_uri))
}

/// Compile an operator-supplied pattern, or `None` when it must not be used as one.
fn compile_anchored(pattern: &str) -> Option<Regex> {
    // The `^…$` spelling is the opt-in signal: without it the value is a literal
    // URI and was already handled by the equality check.
    let inner = pattern.strip_prefix('^')?.strip_suffix('$')?;

    // `\A`/`\z` rather than `^`/`$`: they are unaffected by an inline `(?m)` flag
    // inside the operator's pattern.
    let regex = Regex::new(&format!(r"\A(?:{inner})\z")).ok()?;

    // A pattern accepting the empty string accepts a prefix of everything; it is
    // never a legitimate redirect URI and is how `^/*` behaved.
    if regex.is_match("") {
        return None;
    }

    Some(regex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_literal_matches() {
        assert!(redirect_uri_matches(
            "https://app.example/cb",
            "https://app.example/cb"
        ));
    }

    #[test]
    fn literal_does_not_match_a_uri_containing_it() {
        assert!(!redirect_uri_matches(
            "https://app.example/cb",
            "https://attacker.example/?next=https://app.example/cb"
        ));
    }

    #[test]
    fn dots_in_a_literal_are_not_wildcards() {
        assert!(!redirect_uri_matches(
            "https://app.example/cb",
            "https://appXexample/cb"
        ));
    }

    #[test]
    fn seeded_catch_all_matches_nothing() {
        assert!(!redirect_uri_matches("^/*", "https://attacker.example/x"));
        assert!(!redirect_uri_matches("^/*", ""));
        // …but it still matches itself, so a registered literal is never lost.
        assert!(redirect_uri_matches("^/*", "^/*"));
    }

    #[test]
    fn anchored_pattern_matches_within_its_bounds() {
        let pattern = "^https://app\\.example/.*$";

        assert!(redirect_uri_matches(pattern, "https://app.example/cb"));
        assert!(!redirect_uri_matches(
            pattern,
            "https://attacker.example/#https://app.example/cb"
        ));
    }

    #[test]
    fn unanchored_pattern_is_not_a_pattern() {
        assert!(!redirect_uri_matches(
            "https://app\\.example/.*",
            "https://app.example/cb"
        ));
    }

    #[test]
    fn alternation_cannot_escape_the_anchors() {
        assert!(!redirect_uri_matches(
            "^https://app\\.example/cb|attacker$",
            "https://evil.example/attacker"
        ));
    }

    #[test]
    fn pattern_accepting_the_empty_string_is_refused() {
        assert!(!redirect_uri_matches("^.*$", "https://attacker.example/x"));
        assert!(!redirect_uri_matches("^/*$", "https://attacker.example/x"));
        assert!(!redirect_uri_matches(
            "^(?:)$",
            "https://attacker.example/x"
        ));
    }

    #[test]
    fn multiline_flag_cannot_reopen_the_anchors() {
        assert!(!redirect_uri_matches(
            "^(?m)https://app\\.example/cb$",
            "https://attacker.example/x\nhttps://app.example/cb"
        ));
    }

    #[test]
    fn invalid_regex_matches_nothing() {
        assert!(!redirect_uri_matches("^https://app.example/[$", "anything"));
    }

    #[test]
    fn matches_any_scans_every_registered_value() {
        let registered = ["https://other.example/cb", "^https://app\\.example/.*$"];

        assert!(redirect_uri_matches_any(
            registered,
            "https://app.example/cb"
        ));
        assert!(!redirect_uri_matches_any(
            registered,
            "https://attacker.example/x"
        ));
        assert!(!redirect_uri_matches_any([], "https://app.example/cb"));
    }
}
