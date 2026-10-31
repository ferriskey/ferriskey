#![no_main]

use ferriskey_saml::authn::AuthnRequest;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(xml) = std::str::from_utf8(data) {
        let _ = AuthnRequest::parse(xml);
    }
});
