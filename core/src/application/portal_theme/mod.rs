use crate::{
    ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        portal_layouts::ports::{ImportLayoutInput, PortalLayoutsService},
        portal_theme::{
            entities::{PortalTheme, PortalThemeConfig},
            ports::{
                CreateThemeInput, GetThemeByIdInput, GetThemeInput, ImportPortalThemeInput,
                ListThemesInput, PortalThemeService, UpdateThemeInput, UpdateThemeMetadataInput,
                UpdateThemePageInput,
            },
        },
    },
};

impl PortalThemeService for ApplicationService {
    async fn get_theme(
        &self,
        identity: Identity,
        input: GetThemeInput,
    ) -> Result<PortalThemeConfig, CoreError> {
        self.portal_theme_service.get_theme(identity, input).await
    }

    async fn update_theme(
        &self,
        identity: Identity,
        input: UpdateThemeInput,
    ) -> Result<PortalTheme, CoreError> {
        self.portal_theme_service
            .update_theme(identity, input)
            .await
    }

    async fn get_public_theme(&self, input: GetThemeInput) -> Result<PortalThemeConfig, CoreError> {
        self.portal_theme_service.get_public_theme(input).await
    }

    async fn list_themes(
        &self,
        identity: Identity,
        input: ListThemesInput,
    ) -> Result<Vec<PortalTheme>, CoreError> {
        self.portal_theme_service.list_themes(identity, input).await
    }

    async fn get_theme_by_id(
        &self,
        identity: Identity,
        input: GetThemeByIdInput,
    ) -> Result<PortalTheme, CoreError> {
        self.portal_theme_service
            .get_theme_by_id(identity, input)
            .await
    }

    async fn create_theme(
        &self,
        identity: Identity,
        input: CreateThemeInput,
    ) -> Result<PortalTheme, CoreError> {
        self.portal_theme_service
            .create_theme(identity, input)
            .await
    }

    async fn update_theme_metadata(
        &self,
        identity: Identity,
        input: UpdateThemeMetadataInput,
    ) -> Result<PortalTheme, CoreError> {
        self.portal_theme_service
            .update_theme_metadata(identity, input)
            .await
    }

    async fn update_theme_page(
        &self,
        identity: Identity,
        input: UpdateThemePageInput,
    ) -> Result<PortalTheme, CoreError> {
        self.portal_theme_service
            .update_theme_page(identity, input)
            .await
    }

    async fn activate_theme(
        &self,
        identity: Identity,
        input: GetThemeByIdInput,
    ) -> Result<(), CoreError> {
        self.portal_theme_service
            .activate_theme(identity, input)
            .await
    }

    async fn delete_theme(
        &self,
        identity: Identity,
        input: GetThemeByIdInput,
    ) -> Result<(), CoreError> {
        self.portal_theme_service
            .delete_theme(identity, input)
            .await
    }

    async fn get_active_theme(
        &self,
        input: ListThemesInput,
    ) -> Result<Option<PortalTheme>, CoreError> {
        self.portal_theme_service.get_active_theme(input).await
    }
}

/// Importing a theme spans two domains — the theme and the layout it is framed
/// by — so it is composed here rather than inside either service: the theme
/// service has no layout repository, and giving it one to serve an import
/// would widen it for every other caller.
impl ApplicationService {
    pub async fn import_portal_theme(
        &self,
        identity: Identity,
        input: ImportPortalThemeInput,
    ) -> Result<PortalTheme, CoreError> {
        // The layout travels inside the file, so it is recreated first and the
        // theme is bound to the fresh id. A file exported from a theme with no
        // layout carries none, and the theme keeps the realm's default.
        let layout_id = match input.layout {
            Some(layout) => Some(
                self.import_layout(
                    identity.clone(),
                    ImportLayoutInput {
                        realm_name: input.realm_name.clone(),
                        name: layout.name,
                        tree: layout.tree,
                    },
                )
                .await?
                .id,
            ),
            None => None,
        };

        let theme = self
            .create_theme(
                identity.clone(),
                CreateThemeInput {
                    realm_name: input.realm_name.clone(),
                    name: input.name,
                    layout_id,
                    config: input.config,
                },
            )
            .await?;

        // Pages are written one by one because that is the only write the
        // repository exposes. A page that fails leaves the theme created but
        // incomplete — the caller sees the error, and activation would refuse
        // it anyway, so nothing half-valid can go live.
        for (page_type, tree) in input.pages {
            self.update_theme_page(
                identity.clone(),
                UpdateThemePageInput {
                    realm_name: input.realm_name.clone(),
                    theme_id: theme.id,
                    page_type,
                    tree,
                },
            )
            .await?;
        }

        self.get_theme_by_id(
            identity,
            GetThemeByIdInput {
                realm_name: input.realm_name,
                theme_id: theme.id,
            },
        )
        .await
    }
}
