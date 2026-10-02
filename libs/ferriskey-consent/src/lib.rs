pub mod entities;
pub mod ports;
pub mod services;
pub mod value_objects;

pub use entities::{ConsentDecision, ConsentDecisionId, ScopeDescriptor};
pub use ports::ConsentDecisionRepository;
pub use services::{ConsentService, ConsentServiceImpl};
pub use value_objects::{
    ConsentDecisionOutcome, ConsentEvaluation, ConsentRequestView, DEFAULT_CONSENT_TTL_DAYS,
    DecideConsentInput, EvaluateConsentInput, PendingConsentView,
};
