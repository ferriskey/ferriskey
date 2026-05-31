use chrono::{DateTime, Utc};
use ferriskey_domain::realm::RealmId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct PortalTheme {
    pub id: Uuid,
    pub realm_id: RealmId,
    pub name: String,
    pub layout_id: Option<Uuid>,
    pub config: PortalThemeConfig,
    pub pages: PortalThemePages,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PortalPageType {
    Login,
    Register,
    Totp,
    ForgotPassword,
    ResetPassword,
    MagicLinkVerify,
    /// Page where an unauthenticated user enters their email to receive a
    /// magic link. Distinct from `MagicLinkVerify` (which handles the click
    /// from the email) — this is the request side of the same flow, and
    /// requires at least an `email_input` + a `submit_button`.
    MagicLinkRequest,
    VerifyEmail,
    /// Success screen rendered after a verification link has been clicked
    /// and the email has been confirmed. Distinct from `VerifyEmail`
    /// (which collects / shows the "check your inbox" state) — this is
    /// the post-verification confirmation, typically a "you're verified,
    /// continue to login" page.
    EmailVerified,
    /// First-time TOTP enrolment screen — shows the QR code (and the
    /// fallback secret) that the user scans into their authenticator
    /// app, then captures the resulting 6-digit code + an optional
    /// device label to confirm the binding. Distinct from `Totp` (which
    /// only collects the code on subsequent logins).
    TotpSetup,
}

impl PortalPageType {
    pub const ALL: [PortalPageType; 10] = [
        PortalPageType::Login,
        PortalPageType::Register,
        PortalPageType::Totp,
        PortalPageType::ForgotPassword,
        PortalPageType::ResetPassword,
        PortalPageType::MagicLinkVerify,
        PortalPageType::MagicLinkRequest,
        PortalPageType::VerifyEmail,
        PortalPageType::EmailVerified,
        PortalPageType::TotpSetup,
    ];
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct PortalThemePages {
    pub login: serde_json::Value,
    pub register: serde_json::Value,
    pub totp: serde_json::Value,
    pub forgot_password: serde_json::Value,
    pub reset_password: serde_json::Value,
    pub magic_link_verify: serde_json::Value,
    pub magic_link_request: serde_json::Value,
    pub verify_email: serde_json::Value,
    pub email_verified: serde_json::Value,
    pub totp_setup: serde_json::Value,
}

impl PortalThemePages {
    pub fn get(&self, page_type: PortalPageType) -> &serde_json::Value {
        match page_type {
            PortalPageType::Login => &self.login,
            PortalPageType::Register => &self.register,
            PortalPageType::Totp => &self.totp,
            PortalPageType::ForgotPassword => &self.forgot_password,
            PortalPageType::ResetPassword => &self.reset_password,
            PortalPageType::MagicLinkVerify => &self.magic_link_verify,
            PortalPageType::MagicLinkRequest => &self.magic_link_request,
            PortalPageType::VerifyEmail => &self.verify_email,
            PortalPageType::EmailVerified => &self.email_verified,
            PortalPageType::TotpSetup => &self.totp_setup,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct PortalThemeConfig {
    pub colors: ThemeColors,
    pub fonts: ThemeFonts,
    pub borders: ThemeBorders,
    pub spacing: ThemeSpacing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct ThemeSpacing {
    pub widget_padding: u32,
    pub field_gap: u32,
    pub section_gap: u32,
}

impl Default for ThemeSpacing {
    fn default() -> Self {
        Self {
            widget_padding: 24,
            field_gap: 16,
            section_gap: 24,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct ThemeColors {
    pub primary_button: String,
    pub primary_button_label: String,
    pub secondary_button: String,
    pub secondary_button_label: String,
    /// Background of social auth buttons (Google, GitHub, …). Defaults to
    /// white because most provider icons assume a light surface. Distinct
    /// from `secondary_button` so admins can keep a neutral "Sign in with"
    /// look while branding their primary/secondary CTAs.
    pub social_button_background: String,
    pub social_button_label: String,
    /// Outline color of social buttons. Typically a faint grey — provider
    /// pills tend to read better with a visible border than a flat
    /// background, even when the background matches the page surface.
    pub social_button_border: String,
    /// Magic-link button (alternative auth path). Distinct from the generic
    /// secondary button so admins can keep "Sign in with a magic link"
    /// neutral while branding the primary CTAs separately.
    pub magic_link_button_background: String,
    pub magic_link_button_label: String,
    pub magic_link_button_border: String,
    /// Passkey button (alternative auth path). Same rationale as magic-link.
    pub passkey_button_background: String,
    pub passkey_button_label: String,
    pub passkey_button_border: String,
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
            social_button_background: "#ffffff".to_string(),
            social_button_label: "#1e212a".to_string(),
            social_button_border: "#d1d5db".to_string(),
            magic_link_button_background: "#ffffff".to_string(),
            magic_link_button_label: "#1e212a".to_string(),
            magic_link_button_border: "#d1d5db".to_string(),
            passkey_button_background: "#ffffff".to_string(),
            passkey_button_label: "#1e212a".to_string(),
            passkey_button_border: "#d1d5db".to_string(),
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
    /// Border thickness applied specifically to social auth buttons. Pulled
    /// out from `button_border_weight` so an admin can give provider pills a
    /// thicker outline (the usual visual treatment for OAuth buttons on a
    /// white-on-white surface) without affecting primary/secondary CTAs.
    pub social_button_border_weight: u32,
    /// Border thickness for the Magic link button. Same rationale as
    /// `social_button_border_weight` — alternative auth buttons often want
    /// a different visual weight than primary/secondary CTAs.
    pub magic_link_button_border_weight: u32,
    /// Border thickness for the Passkey button.
    pub passkey_button_border_weight: u32,
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
            social_button_border_weight: 1,
            magic_link_button_border_weight: 1,
            passkey_button_border_weight: 1,
            input_radius: 3,
            input_border_weight: 1,
            widget_radius: 5,
            // 1px border combined with the soft shadow gives the card a
            // distinct edge instead of relying on the shadow alone — the
            // FerrisKey default design pairs a faint outline with a soft
            // elevation, so we ship that as the out-of-box look.
            widget_border_weight: 1,
            // Default to the pronounced shadow so the widget card reads as
            // an elevated surface out of the box — matches the look of the
            // FerrisKey admin's default login screen. Admins can dial back
            // to `Small` or `None` from Theme → Widget → Shadow.
            widget_shadow: ThemeShadow::Large,
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
