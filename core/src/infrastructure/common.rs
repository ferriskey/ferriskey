/// Tells a unique-constraint violation apart from a genuine server fault.
///
/// A schema-level uniqueness conflict is a caller asking for a value that is
/// taken, not a failure of the server. Matching on the SQLSTATE rather than the
/// message keeps this independent of the constraint's name and of the server's
/// locale.
pub fn is_unique_violation(error: &sea_orm::DbErr) -> bool {
    matches!(
        error.sql_err(),
        Some(sea_orm::SqlErr::UniqueConstraintViolation(_))
    )
}
