//! Domain model for the OAuth 2.0 Device Authorization Grant (RFC 8628).

pub mod entities;
pub mod ports;
pub mod value_objects;

pub use entities::{DeviceAuthSession, DeviceAuthSessionConfig, DeviceAuthStatus, UserCode};
pub use ports::DeviceAuthRepository;
pub use value_objects::{
    InitiateDeviceFlowInput, InitiateDeviceFlowOutput, PollDeviceTokenInput, VerifyUserCodeInput,
};
