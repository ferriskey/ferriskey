use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::{
        common::{entities::app_errors::CoreError, generate_timestamp},
        email_template::{entities::EmailTemplate, ports::EmailTemplateRepository},
    },
    entity::email_templates::{
        ActiveModel as EmailTemplateActiveModel, Column as EmailTemplateColumn,
        Entity as EmailTemplateEntity,
    },
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

impl EmailTemplateRepository for PostgresEmailTemplateRepository {
    async fn fetch_by_realm(&self, realm_id: Uuid) -> Result<Vec<EmailTemplate>, CoreError> {
        EmailTemplateEntity::find()
            .filter(EmailTemplateColumn::RealmId.eq(realm_id))
            .all(&self.db)
            .await
            .map(|models| models.into_iter().map(EmailTemplate::from).collect())
            .map_err(|e| {
                error!("Failed to fetch email templates: {}", e);
                CoreError::InternalServerError
            })
    }

    async fn get_by_id(&self, template_id: Uuid) -> Result<Option<EmailTemplate>, CoreError> {
        EmailTemplateEntity::find_by_id(template_id)
            .one(&self.db)
            .await
            .map(|model| model.map(EmailTemplate::from))
            .map_err(|e| {
                error!("Failed to get email template: {}", e);
                CoreError::InternalServerError
            })
    }

    async fn get_active_by_type(
        &self,
        realm_id: Uuid,
        email_type: String,
    ) -> Result<Option<EmailTemplate>, CoreError> {
        EmailTemplateEntity::find()
            .filter(EmailTemplateColumn::RealmId.eq(realm_id))
            .filter(EmailTemplateColumn::EmailType.eq(&email_type))
            .filter(EmailTemplateColumn::IsActive.eq(true))
            .one(&self.db)
            .await
            .map(|model| model.map(EmailTemplate::from))
            .map_err(|e| {
                error!("Failed to get active email template: {}", e);
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
            is_active: Set(false),
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
        template_id: Uuid,
        name: String,
        structure: serde_json::Value,
        mjml: String,
    ) -> Result<EmailTemplate, CoreError> {
        let existing = EmailTemplateEntity::find_by_id(template_id)
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

    async fn delete(&self, template_id: Uuid) -> Result<(), CoreError> {
        EmailTemplateEntity::delete_by_id(template_id)
            .exec(&self.db)
            .await
            .map(|_| ())
            .map_err(|e| {
                error!("Failed to delete email template: {}", e);
                CoreError::InternalServerError
            })
    }

    async fn activate(
        &self,
        realm_id: Uuid,
        email_type: String,
        template_id: Uuid,
    ) -> Result<EmailTemplate, CoreError> {
        // Deactivate all templates of same type in this realm
        let all_active = EmailTemplateEntity::find()
            .filter(EmailTemplateColumn::RealmId.eq(realm_id))
            .filter(EmailTemplateColumn::EmailType.eq(&email_type))
            .filter(EmailTemplateColumn::IsActive.eq(true))
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to fetch active templates: {}", e);
                CoreError::InternalServerError
            })?;

        for model in all_active {
            let mut active: EmailTemplateActiveModel = model.into();
            active.is_active = Set(false);
            active.updated_at = Set(chrono::Utc::now().naive_utc());
            active.update(&self.db).await.map_err(|e| {
                error!("Failed to deactivate template: {}", e);
                CoreError::InternalServerError
            })?;
        }

        // Activate the target template
        let target = EmailTemplateEntity::find_by_id(template_id)
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to find template to activate: {}", e);
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::EmailTemplateNotFound)?;

        let mut active: EmailTemplateActiveModel = target.into();
        active.is_active = Set(true);
        active.updated_at = Set(chrono::Utc::now().naive_utc());

        active
            .update(&self.db)
            .await
            .map(EmailTemplate::from)
            .map_err(|e| {
                error!("Failed to activate template: {}", e);
                CoreError::InternalServerError
            })
    }
}
