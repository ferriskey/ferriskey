#![no_main]

use ferriskey_saml::authn::AuthnRequest;
use ferriskey_saml::binding::decode_redirect;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(message) = std::str::from_utf8(data)
        && let Ok(xml) = decode_redirect(message)
    {
        let _ = AuthnRequest::parse(&xml);
    }
});
