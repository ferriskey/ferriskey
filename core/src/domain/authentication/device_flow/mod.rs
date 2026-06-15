//! Domain model for the OAuth 2.0 Device Authorization Grant (RFC 8628).

pub mod entities;
pub mod error;
pub mod ports;
pub mod services;
pub mod value_objects;

pub use entities::{
    DeviceAuthSession, DeviceAuthSessionConfig, DeviceAuthStatus, DeviceFlowEventPayload, UserCode,
};
pub use error::DeviceFlowError;
pub use ports::{DeviceAuthRepository, DeviceFlowService, DeviceTokenIssuer};
pub use services::{DeviceFlowConfig, DeviceFlowServiceImpl};
pub use value_objects::{
    InitiateDeviceFlowInput, InitiateDeviceFlowOutput, InitiateDeviceFlowParams,
    PollDeviceTokenInput, PollDeviceTokenParams, VerifyUserCodeInput,
};
