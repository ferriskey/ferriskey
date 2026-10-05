use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryTrait,
    Select, Set,
};
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::{
        common::{
            entities::app_errors::CoreError,
            generate_timestamp,
            pagination::{Page, PageRequest},
        },
        email_template::{
            entities::{EmailTemplate, EmailTemplateFilter, EmailTemplateSortField},
            ports::EmailTemplateRepository,
        },
        realm::entities::{RealmScope, Scoped, Unscoped},
    },
    entity::email_templates::{
        ActiveModel as EmailTemplateActiveModel, Column as EmailTemplateColumn,
        Entity as EmailTemplateEntity,
    },
    infrastructure::pagination::{SortColumn, contains, paginate},
};

#[derive(Debug, Clone)]
pub struct PostgresEmailTemplateRepository {
    pub db: DatabaseConnection,
}

impl PostgresEmailTemplateRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl SortColumn<EmailTemplateEntity> for EmailTemplateSortField {
    fn column(&self) -> EmailTemplateColumn {
        match self {
            EmailTemplateSortField::Name => EmailTemplateColumn::Name,
            EmailTemplateSortField::EmailType => EmailTemplateColumn::EmailType,
            EmailTemplateSortField::CreatedAt => EmailTemplateColumn::CreatedAt,
            EmailTemplateSortField::UpdatedAt => EmailTemplateColumn::UpdatedAt,
        }
    }
}

fn listing_select(realm_id: Uuid, filter: &EmailTemplateFilter) -> Select<EmailTemplateEntity> {
    EmailTemplateEntity::find()
        .filter(EmailTemplateColumn::RealmId.eq(realm_id))
        .apply_if(filter.name.as_deref(), |select, value| {
            select.filter(contains(EmailTemplateColumn::Name, value))
        })
        .apply_if(filter.email_type.as_ref(), |select, value| {
            select.filter(EmailTemplateColumn::EmailType.eq(value.to_string()))
        })
}

impl EmailTemplateRepository for PostgresEmailTemplateRepository {
    async fn list(
        &self,
        scope: &RealmScope,
        request: &PageRequest<EmailTemplateFilter, EmailTemplateSortField>,
    ) -> Result<Page<EmailTemplate>, CoreError> {
        let (models, total) = paginate(
            &self.db,
            listing_select(scope.id().into(), &request.filter),
            request,
        )
        .await
        .map_err(|e| {
            error!("Failed to list email templates: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(Page::new(
            models.into_iter().map(EmailTemplate::from).collect(),
            total,
            request.page,
            request.limit,
        ))
    }

    async fn get_by_id(
        &self,
        realm_id: Uuid,
        template_id: Uuid,
    ) -> Result<Option<Unscoped<EmailTemplate>>, CoreError> {
        EmailTemplateEntity::find_by_id(template_id)
            .filter(EmailTemplateColumn::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map(|model| model.map(|model| Unscoped::new(EmailTemplate::from(model))))
            .map_err(|e| {
                error!("Failed to get email template: {}", e);
                CoreError::InternalServerError
            })
    }

    async fn create(
        &self,
        realm_id: Uuid,
        name: String,
        email_type: String,
        structure: serde_json::Value,
        mjml: String,
    ) -> Result<EmailTemplate, CoreError> {
        let (_, timestamp) = generate_timestamp();
        let id = Uuid::new_v7(timestamp);

        let model = EmailTemplateActiveModel {
            id: Set(id),
            realm_id: Set(realm_id),
            name: Set(name),
            email_type: Set(email_type),
            structure: Set(structure),
            mjml: Set(mjml),
            created_at: Set(chrono::Utc::now().naive_utc()),
            updated_at: Set(chrono::Utc::now().naive_utc()),
        };

        EmailTemplateEntity::insert(model)
            .exec_with_returning(&self.db)
            .await
            .map(EmailTemplate::from)
            .map_err(|e| {
                error!("Failed to create email template: {}", e);
                CoreError::InternalServerError
            })
    }

    async fn update(
        &self,
        template: &Scoped<EmailTemplate>,
        name: String,
        structure: serde_json::Value,
        mjml: String,
    ) -> Result<EmailTemplate, CoreError> {
        let existing = EmailTemplateEntity::find_by_id(template.get().id)
            .filter(EmailTemplateColumn::RealmId.eq(template.get().realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to find email template for update: {}", e);
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::EmailTemplateNotFound)?;

        let mut active: EmailTemplateActiveModel = existing.into();
        active.name = Set(name);
        active.structure = Set(structure);
        active.mjml = Set(mjml);
        active.updated_at = Set(chrono::Utc::now().naive_utc());

        active
            .update(&self.db)
            .await
            .map(EmailTemplate::from)
            .map_err(|e| {
                error!("Failed to update email template: {}", e);
                CoreError::InternalServerError
            })
    }

    async fn delete(&self, template: &Scoped<EmailTemplate>) -> Result<(), CoreError> {
        EmailTemplateEntity::delete_many()
            .filter(EmailTemplateColumn::Id.eq(template.get().id))
            .filter(EmailTemplateColumn::RealmId.eq(template.get().realm_id))
            .exec(&self.db)
            .await
            .map(|_| ())
            .map_err(|e| {
                error!("Failed to delete email template: {}", e);
                CoreError::InternalServerError
            })
    }
}

#[cfg(test)]
mod listing_tests {
    use sea_orm::{DbBackend, QueryTrait};
    use uuid::Uuid;

    use super::listing_select;
    use crate::domain::email_template::entities::{EmailTemplateFilter, EmailType};

    fn sql(filter: &EmailTemplateFilter) -> String {
        listing_select(Uuid::nil(), filter)
            .build(DbBackend::Postgres)
            .to_string()
    }

    #[test]
    fn listing_is_always_bound_to_the_realm() {
        let sql = sql(&EmailTemplateFilter::default());
        assert!(
            sql.contains(
                r#""email_templates"."realm_id" = '00000000-0000-0000-0000-000000000000'"#
            ),
            "{sql}"
        );
        assert!(!sql.contains("ILIKE"), "{sql}");
        assert!(!sql.contains(r#""email_type" ="#), "{sql}");
    }

    #[test]
    fn name_is_an_escaped_contains_match() {
        let sql = sql(&EmailTemplateFilter {
            name: Some("a%".to_string()),
            ..EmailTemplateFilter::default()
        });
        assert!(
            sql.contains(r#""email_templates"."name" ILIKE E'%a\\%%'"#),
            "{sql}"
        );
    }

    #[test]
    fn email_type_is_an_equality_on_its_stored_name() {
        let sql = sql(&EmailTemplateFilter {
            email_type: Some(EmailType::EmailVerification),
            ..EmailTemplateFilter::default()
        });
        assert!(
            sql.contains(r#""email_templates"."email_type" = 'email_verification'"#),
            "{sql}"
        );
    }
}
