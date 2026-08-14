pub mod services;

pub use ferriskey_aegis::entities;
pub use ferriskey_aegis::ports;
pub use ferriskey_aegis::value_objects;

/// The aegis repository ports are `automock`ed in `ferriskey-aegis` behind its `mock`
/// feature, which `core` enables. This module used to re-declare all four of them in
/// hand-written `mockall::mock!` blocks; every port signature change then had to be
/// copied here by hand, and a stale copy failed to compile long after the real trait
/// had moved on. Re-export the generated mocks instead.
#[cfg(test)]
pub mod mocks {
    pub use ferriskey_aegis::ports::{
        MockClientScopeAttributeRepository, MockClientScopeMappingRepository,
        MockClientScopeRepository, MockProtocolMapperRepository,
    };
}
