use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use utoipa::{
    PartialSchema, ToSchema,
    openapi::schema::{SchemaType, Type},
    openapi::{ObjectBuilder, RefOr, Schema},
};

use crate::strategies::{FullMask, MaskStrategy};

/// What gets produced when data is serialized / logged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Redaction {
    /// Always "***"
    Full,
    /// Strategy-based output (partial, email, hash...)
    Strategy,
}

/// Generic sensitive wrapper.
///
/// - `Deserialize`: stores the real value
/// - `Serialize`: outputs redacted string (never the real value)
/// - `Debug` / `Display`: redacted
#[derive(Clone, Eq, PartialEq)]
pub struct Masked<T> {
    value: T,
    mode: Redaction,
}

impl<T> Masked<T> {
    /// Store sensitive value and choose redaction mode.
    pub fn new(value: T) -> Self {
        Self {
            value,
            mode: Redaction::Full,
        }
    }

    /// Store sensitive value and choose redaction mode.
    pub fn with_mode(value: T, mode: Redaction) -> Self {
        Self { value, mode }
    }

    /// Explicit access. Keep this name intentionally "loud".
    pub fn expose(&self) -> &T {
        &self.value
    }

    pub fn into_inner(self) -> T {
        self.value
    }

    pub fn mode(&self) -> Redaction {
        self.mode
    }
}

impl<T> Debug for Masked<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Never leak
        f.debug_struct("Masked").field("value", &"***").finish()
    }
}

impl<T> Display for Masked<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Never leak
        write!(f, "***")
    }
}

impl<T> Serialize for Masked<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Generic type: always "***"
        serializer.serialize_str("***")
    }
}

impl<'de, T> Deserialize<'de> for Masked<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer).map(Masked::new)
    }
}

impl<T> ToSchema for Masked<T> {
    fn name() -> Cow<'static, str> {
        Cow::Borrowed("Masked")
    }
}

impl<T> PartialSchema for Masked<T> {
    fn schema() -> RefOr<Schema> {
        RefOr::T(Schema::Object(
            ObjectBuilder::new()
                .schema_type(SchemaType::Type(Type::String))
                .build(),
        ))
    }
}

/// String-specialized wrapper where redaction can be strategy-driven.
/// (Partial/email/hash etc.)
#[derive(Clone, Eq, PartialEq)]
pub struct MaskedWith<S: MaskStrategy> {
    value: String,
    mode: Redaction,
    _strategy: core::marker::PhantomData<S>,
}

impl<S: MaskStrategy> MaskedWith<S> {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            mode: Redaction::Strategy,
            _strategy: core::marker::PhantomData,
        }
    }

    pub fn expose(&self) -> &str {
        &self.value
    }

    pub fn into_inner(self) -> String {
        self.value
    }

    pub fn mode(&self) -> Redaction {
        self.mode
    }
}

impl<S: MaskStrategy> Debug for MaskedWith<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MaskedWith")
            .field("value", &S::mask(&self.value))
            .finish()
    }
}

impl<S: MaskStrategy> Display for MaskedWith<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", S::mask(&self.value))
    }
}

impl<S: MaskStrategy> Serialize for MaskedWith<S> {
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        match self.mode {
            Redaction::Full => serializer.serialize_str("***"),
            Redaction::Strategy => serializer.serialize_str(&S::mask(&self.value)),
        }
    }
}

impl<'de, S: MaskStrategy> Deserialize<'de> for MaskedWith<S> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new(s))
    }
}

impl<S: MaskStrategy> ToSchema for MaskedWith<S> {
    fn name() -> Cow<'static, str> {
        Cow::Borrowed("MaskedWith")
    }
}

impl<S: MaskStrategy> PartialSchema for MaskedWith<S> {
    fn schema() -> RefOr<Schema> {
        RefOr::T(Schema::Object(
            ObjectBuilder::new()
                .schema_type(SchemaType::Type(Type::String))
                .build(),
        ))
    }
}

pub type MaskedString = MaskedWith<FullMask>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use utoipa::{PartialSchema, ToSchema};
    use utoipa::openapi::RefOr;
    use utoipa::openapi::schema::{SchemaType, Type};

    #[test]
    fn masked_redacts_in_display_debug_and_serde() {
        let masked = Masked::new("secret".to_string());

        assert_eq!(masked.expose(), "secret");
        assert_eq!(format!("{}", masked), "***");
        assert!(format!("{:?}", masked).contains("***"));
        assert_eq!(serde_json::to_string(&masked).unwrap(), "\"***\"");
    }

    #[test]
    fn masked_deserializes_original_value() {
        let masked: Masked<String> = serde_json::from_str("\"secret\"").unwrap();
        assert_eq!(masked.expose(), "secret");
    }

    #[test]
    fn masked_with_strategy_serializes_masked_value() {
        let masked = MaskedWith::<crate::strategies::EmailMask>::new("user@example.com");
        assert_eq!(masked.expose(), "user@example.com");
        assert_eq!(format!("{}", masked), "******@example.com");
        assert!(format!("{:?}", masked).contains("******@example.com"));
        assert_eq!(
            serde_json::to_string(&masked).unwrap(),
            "\"******@example.com\""
        );
    }

    #[test]
    fn masked_with_deserializes_original_value() {
        let masked: MaskedWith<crate::strategies::FullMask> =
            serde_json::from_str("\"secret\"").unwrap();
        assert_eq!(masked.expose(), "secret");
    }

    #[test]
    fn masked_schema_is_string() {
        let schema = <Masked<String> as PartialSchema>::schema();
        match schema {
            RefOr::T(utoipa::openapi::Schema::Object(object)) => {
                assert!(matches!(
                    object.schema_type,
                    SchemaType::Type(Type::String)
                ));
            }
            _ => panic!("Expected object schema"),
        }
        assert_eq!(<Masked<String> as ToSchema>::name(), Cow::Borrowed("Masked"));
    }

    #[test]
    fn masked_with_schema_is_string() {
        let schema = <MaskedWith<crate::strategies::FullMask> as PartialSchema>::schema();
        match schema {
            RefOr::T(utoipa::openapi::Schema::Object(object)) => {
                assert!(matches!(
                    object.schema_type,
                    SchemaType::Type(Type::String)
                ));
            }
            _ => panic!("Expected object schema"),
        }
        assert_eq!(
            <MaskedWith<crate::strategies::FullMask> as ToSchema>::name(),
            Cow::Borrowed("MaskedWith")
        );
    }
}
