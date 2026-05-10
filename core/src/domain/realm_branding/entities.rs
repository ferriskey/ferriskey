use chrono::{DateTime, Utc};
use ferriskey_domain::realm::RealmId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct RealmBranding {
    pub id: Uuid,
    pub realm_id: RealmId,
    pub config: BrandingConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct BrandingConfig {
    pub colors: BrandingColors,
    pub fonts: BrandingFonts,
    pub borders: BrandingBorders,
    pub widget: BrandingWidget,
    pub page: BrandingPage,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct BrandingColors {
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

impl Default for BrandingColors {
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
pub struct BrandingFonts {
    pub url: Option<String>,
    pub base_size: u32,
    pub title: BrandingFontStyle,
    pub subtitle: BrandingFontStyle,
    pub body: BrandingFontStyle,
    pub buttons: BrandingFontStyle,
    pub input_labels: BrandingFontStyle,
    pub links: BrandingFontLinkStyle,
}

impl Default for BrandingFonts {
    fn default() -> Self {
        Self {
            url: None,
            base_size: 16,
            title: BrandingFontStyle {
                weight: 600,
                size_pct: 150.0,
            },
            subtitle: BrandingFontStyle {
                weight: 400,
                size_pct: 87.5,
            },
            body: BrandingFontStyle {
                weight: 400,
                size_pct: 87.5,
            },
            buttons: BrandingFontStyle {
                weight: 600,
                size_pct: 100.0,
            },
            input_labels: BrandingFontStyle {
                weight: 500,
                size_pct: 100.0,
            },
            links: BrandingFontLinkStyle {
                weight: 600,
                size_pct: 87.5,
                style: BrandingLinkStyle::Normal,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BrandingFontStyle {
    pub weight: u32,
    pub size_pct: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BrandingFontLinkStyle {
    pub weight: u32,
    pub size_pct: f32,
    pub style: BrandingLinkStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum BrandingLinkStyle {
    Normal,
    Underline,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct BrandingBorders {
    pub button_radius: u32,
    pub button_border_weight: u32,
    pub input_radius: u32,
    pub input_border_weight: u32,
    pub widget_radius: u32,
    pub widget_border_weight: u32,
    pub widget_shadow: BrandingShadow,
}

impl Default for BrandingBorders {
    fn default() -> Self {
        Self {
            button_radius: 3,
            button_border_weight: 1,
            input_radius: 3,
            input_border_weight: 1,
            widget_radius: 5,
            widget_border_weight: 0,
            widget_shadow: BrandingShadow::Small,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum BrandingShadow {
    None,
    Small,
    Large,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct BrandingWidget {
    pub logo_url: Option<String>,
    pub logo_height: u32,
    pub header_alignment: BrandingAlignment,
    pub social_buttons_layout: SocialButtonsLayout,
}

impl Default for BrandingWidget {
    fn default() -> Self {
        Self {
            logo_url: None,
            logo_height: 52,
            header_alignment: BrandingAlignment::Center,
            social_buttons_layout: SocialButtonsLayout::Icons,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum BrandingAlignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SocialButtonsLayout {
    Icons,
    List,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct BrandingPage {
    pub favicon_url: Option<String>,
    pub background_color: String,
    pub background_image_url: Option<String>,
}

impl Default for BrandingPage {
    fn default() -> Self {
        Self {
            favicon_url: None,
            background_color: "#000000".to_string(),
            background_image_url: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_round_trips_through_json() {
        let original = BrandingConfig::default();
        let json = serde_json::to_string(&original).expect("serialize");
        let parsed: BrandingConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, parsed);
    }

    #[test]
    fn partial_json_fills_with_defaults() {
        let json = r##"{ "colors": { "primaryButton": "#ff0000" } }"##;
        let parsed: BrandingConfig = serde_json::from_str(json).expect("deserialize");
        assert_eq!(parsed.colors.primary_button, "#ff0000");
        assert_eq!(
            parsed.colors.primary_button_label,
            BrandingColors::default().primary_button_label
        );
        assert_eq!(parsed.fonts, BrandingFonts::default());
        assert_eq!(parsed.borders, BrandingBorders::default());
        assert_eq!(parsed.widget, BrandingWidget::default());
        assert_eq!(parsed.page, BrandingPage::default());
    }

    #[test]
    fn empty_object_is_valid_default() {
        let parsed: BrandingConfig = serde_json::from_str("{}").expect("deserialize");
        assert_eq!(parsed, BrandingConfig::default());
    }

    #[test]
    fn camel_case_keys_are_used_in_json() {
        let cfg = BrandingConfig::default();
        let json = serde_json::to_value(&cfg).expect("to_value");
        assert!(json.get("colors").unwrap().get("primaryButton").is_some());
        assert!(
            json.get("widget")
                .unwrap()
                .get("socialButtonsLayout")
                .is_some()
        );
        assert!(json.get("page").unwrap().get("backgroundColor").is_some());
    }
}
