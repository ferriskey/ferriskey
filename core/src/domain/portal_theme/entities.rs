use chrono::{DateTime, Utc};
use ferriskey_domain::realm::RealmId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct PortalTheme {
    pub id: Uuid,
    pub realm_id: RealmId,
    pub config: PortalThemeConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct PortalThemeConfig {
    pub colors: ThemeColors,
    pub fonts: ThemeFonts,
    pub borders: ThemeBorders,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct ThemeColors {
    pub primary_button: String,
    pub primary_button_label: String,
    pub secondary_button: String,
    pub secondary_button_label: String,
    pub widget_background: String,
    pub page_background: String,
    pub body_text: String,
    pub links: String,
    pub error: String,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            primary_button: "#635dff".to_string(),
            primary_button_label: "#ffffff".to_string(),
            secondary_button: "#ffffff".to_string(),
            secondary_button_label: "#1e212a".to_string(),
            widget_background: "#ffffff".to_string(),
            page_background: "#000000".to_string(),
            body_text: "#1e212a".to_string(),
            links: "#635dff".to_string(),
            error: "#d03c38".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct ThemeFonts {
    pub url: Option<String>,
    pub base_size: u32,
    pub title: ThemeFontStyle,
    pub subtitle: ThemeFontStyle,
    pub body: ThemeFontStyle,
    pub buttons: ThemeFontStyle,
    pub input_labels: ThemeFontStyle,
    pub links: ThemeFontLinkStyle,
}

impl Default for ThemeFonts {
    fn default() -> Self {
        Self {
            url: None,
            base_size: 16,
            title: ThemeFontStyle {
                weight: 600,
                size_pct: 150.0,
            },
            subtitle: ThemeFontStyle {
                weight: 400,
                size_pct: 87.5,
            },
            body: ThemeFontStyle {
                weight: 400,
                size_pct: 87.5,
            },
            buttons: ThemeFontStyle {
                weight: 600,
                size_pct: 100.0,
            },
            input_labels: ThemeFontStyle {
                weight: 500,
                size_pct: 100.0,
            },
            links: ThemeFontLinkStyle {
                weight: 600,
                size_pct: 87.5,
                style: ThemeLinkStyle::Normal,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThemeFontStyle {
    pub weight: u32,
    pub size_pct: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThemeFontLinkStyle {
    pub weight: u32,
    pub size_pct: f32,
    pub style: ThemeLinkStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ThemeLinkStyle {
    Normal,
    Underline,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct ThemeBorders {
    pub button_radius: u32,
    pub button_border_weight: u32,
    pub input_radius: u32,
    pub input_border_weight: u32,
    pub widget_radius: u32,
    pub widget_border_weight: u32,
    pub widget_shadow: ThemeShadow,
}

impl Default for ThemeBorders {
    fn default() -> Self {
        Self {
            button_radius: 3,
            button_border_weight: 1,
            input_radius: 3,
            input_border_weight: 1,
            widget_radius: 5,
            widget_border_weight: 0,
            widget_shadow: ThemeShadow::Small,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ThemeShadow {
    None,
    Small,
    Large,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_round_trips_through_json() {
        let original = PortalThemeConfig::default();
        let json = serde_json::to_string(&original).expect("serialize");
        let parsed: PortalThemeConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, parsed);
    }

    #[test]
    fn partial_json_fills_with_defaults() {
        let json = r##"{ "colors": { "primaryButton": "#ff0000" } }"##;
        let parsed: PortalThemeConfig = serde_json::from_str(json).expect("deserialize");
        assert_eq!(parsed.colors.primary_button, "#ff0000");
        assert_eq!(
            parsed.colors.primary_button_label,
            ThemeColors::default().primary_button_label
        );
        assert_eq!(parsed.fonts, ThemeFonts::default());
        assert_eq!(parsed.borders, ThemeBorders::default());
    }

    #[test]
    fn empty_object_is_valid_default() {
        let parsed: PortalThemeConfig = serde_json::from_str("{}").expect("deserialize");
        assert_eq!(parsed, PortalThemeConfig::default());
    }

    #[test]
    fn camel_case_keys_are_used_in_json() {
        let cfg = PortalThemeConfig::default();
        let json = serde_json::to_value(&cfg).expect("to_value");
        assert!(json.get("colors").unwrap().get("primaryButton").is_some());
        assert!(json.get("borders").unwrap().get("buttonRadius").is_some());
        assert!(json.get("fonts").unwrap().get("baseSize").is_some());
    }

    #[test]
    fn ignores_unknown_legacy_widget_and_page_fields() {
        // Legacy realm_branding rows had widget+page sections; deserialize must drop them.
        let legacy = r##"{
            "colors": { "primaryButton": "#abc123" },
            "widget": { "logoUrl": "https://x", "logoHeight": 80 },
            "page": { "faviconUrl": null, "backgroundColor": "#fff" }
        }"##;
        let parsed: PortalThemeConfig = serde_json::from_str(legacy).expect("deserialize");
        assert_eq!(parsed.colors.primary_button, "#abc123");
    }
}
