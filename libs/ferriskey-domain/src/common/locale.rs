use std::fmt;

use thiserror::Error;

const EN_TAG: &str = "en";
const SUBTAG_LENGTH: usize = 2;
const SUBTAG_SEPARATOR: char = '-';

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LocaleError {
    #[error("`{0}` is not a well-formed language tag")]
    Malformed(String),

    #[error("`{0}` is not among the supported locales")]
    Unsupported(String),

    #[error("a supported locale set carries at least one locale")]
    EmptySupportedSet,

    #[error("the default locale is absent from the supported locales")]
    DefaultNotSupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locale(String);

impl Locale {
    pub fn en() -> Self {
        Self(EN_TAG.to_string())
    }

    pub fn parse(raw: &str) -> Result<Self, LocaleError> {
        let tag = raw.trim();

        let (language, region) = match tag.split_once(SUBTAG_SEPARATOR) {
            Some((language, region)) => (language, Some(region)),
            None => (tag, None),
        };

        if !is_subtag(language) {
            return Err(LocaleError::Malformed(tag.to_string()));
        }

        match region {
            None => Ok(Self(language.to_ascii_lowercase())),
            Some(region) if is_subtag(region) => {
                let language = language.to_ascii_lowercase();
                let region = region.to_ascii_uppercase();

                Ok(Self(format!("{language}{SUBTAG_SEPARATOR}{region}")))
            }
            Some(_) => Err(LocaleError::Malformed(tag.to_string())),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_subtag(value: &str) -> bool {
    value.len() == SUBTAG_LENGTH && value.bytes().all(|byte| byte.is_ascii_alphabetic())
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportedLocales {
    default: Locale,
    all: Vec<Locale>,
}

impl SupportedLocales {
    pub fn new(default: Locale, all: Vec<Locale>) -> Result<Self, LocaleError> {
        if all.is_empty() {
            return Err(LocaleError::EmptySupportedSet);
        }

        let mut unique: Vec<Locale> = Vec::with_capacity(all.len());
        for locale in all {
            if !unique.contains(&locale) {
                unique.push(locale);
            }
        }

        if !unique.contains(&default) {
            return Err(LocaleError::DefaultNotSupported);
        }

        Ok(Self {
            default,
            all: unique,
        })
    }

    pub fn default_locale(&self) -> &Locale {
        &self.default
    }

    pub fn all(&self) -> &[Locale] {
        &self.all
    }

    pub fn contains(&self, locale: &Locale) -> bool {
        self.all.contains(locale)
    }

    pub fn resolve(&self, requested: Option<&Locale>) -> &Locale {
        requested
            .and_then(|locale| self.all.iter().find(|supported| *supported == locale))
            .unwrap_or(&self.default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn locale(raw: &str) -> Locale {
        Locale::parse(raw).expect("test fixture must be a well-formed language tag")
    }

    fn supported(default: &str, all: &[&str]) -> SupportedLocales {
        SupportedLocales::new(locale(default), all.iter().map(|raw| locale(raw)).collect())
            .expect("test fixture must be a valid supported set")
    }

    #[test]
    fn the_source_locale_is_english() {
        assert_eq!(Locale::en(), locale(EN_TAG));
        assert_eq!(Locale::en().as_str(), "en");
    }

    #[test]
    fn the_whole_supported_set_is_readable_in_its_declared_order() {
        let set = supported("en", &["en", "zh-CN"]);

        assert_eq!(set.all(), &[locale("en"), locale("zh-CN")]);
        assert!(set.all().contains(set.default_locale()));
    }

    #[test]
    fn a_bare_language_round_trips_through_its_tag() {
        assert_eq!(locale("en").as_str(), "en");
        assert_eq!(locale(locale("en").as_str()), locale("en"));
    }

    #[test]
    fn a_language_and_region_round_trip_through_their_tag() {
        assert_eq!(locale("zh-CN").as_str(), "zh-CN");
        assert_eq!(locale(locale("zh-CN").as_str()), locale("zh-CN"));
    }

    #[test]
    fn parsing_lowercases_the_language_and_uppercases_the_region() {
        assert_eq!(locale("ZH-cn").as_str(), "zh-CN");
        assert_eq!(locale("zh-cn").as_str(), "zh-CN");
        assert_eq!(locale("EN").as_str(), "en");
    }

    #[test]
    fn tags_differing_only_by_case_are_the_same_locale() {
        assert_eq!(locale("zh-cn"), locale("ZH-CN"));
        assert_eq!(locale("zh-cn"), locale("zh-CN"));
        assert_eq!(locale("ZH-cn"), locale("zH-Cn"));
    }

    #[test]
    fn a_mixed_case_tag_round_trips_through_its_normalized_form() {
        assert_eq!(locale(locale("ZH-cn").as_str()), locale("zh-CN"));
    }

    #[test]
    fn a_malformed_tag_is_rejected() {
        for raw in [
            "english", "zh_CN", "e", "en-", "", "-", "-CN", "en-USA", "eng-CN", "e1", "12",
            "en-C1", "en-CN-x", "zh CN",
        ] {
            assert_eq!(
                Locale::parse(raw),
                Err(LocaleError::Malformed(raw.to_string())),
                "`{raw}` must not parse"
            );
        }
    }

    #[test]
    fn surrounding_whitespace_does_not_change_a_tag() {
        assert_eq!(locale("  zh-CN  "), locale("zh-CN"));
    }

    #[test]
    fn a_blank_tag_is_rejected() {
        assert_eq!(
            Locale::parse("   "),
            Err(LocaleError::Malformed(String::new()))
        );
    }

    #[test]
    fn a_locale_renders_as_its_normalized_tag() {
        assert_eq!(locale("ZH-cn").to_string(), "zh-CN");
    }

    #[test]
    fn an_empty_supported_set_is_rejected() {
        assert_eq!(
            SupportedLocales::new(Locale::en(), Vec::new()),
            Err(LocaleError::EmptySupportedSet)
        );
    }

    #[test]
    fn a_default_absent_from_the_supported_set_is_rejected() {
        assert_eq!(
            SupportedLocales::new(locale("en"), vec![locale("zh-CN")]),
            Err(LocaleError::DefaultNotSupported)
        );
    }

    #[test]
    fn a_default_written_in_another_case_still_belongs_to_the_set() {
        let set = SupportedLocales::new(locale("ZH-cn"), vec![locale("zh-CN"), locale("en")])
            .expect("case is normalized before membership is checked");

        assert_eq!(set.default_locale(), &locale("zh-CN"));
    }

    #[test]
    fn duplicates_collapse_while_the_declared_order_is_kept() {
        let set = supported("en", &["en", "zh-CN", "en", "ZH-cn"]);

        assert_eq!(set, supported("en", &["en", "zh-CN"]));
    }

    #[test]
    fn membership_answers_for_both_the_default_and_the_others() {
        let set = supported("en", &["en", "zh-CN"]);

        assert!(set.contains(&locale("en")));
        assert!(set.contains(&locale("zh-CN")));
        assert!(!set.contains(&locale("fr-FR")));
    }

    #[test]
    fn resolving_nothing_yields_the_default() {
        let set = supported("en", &["en", "zh-CN"]);

        assert_eq!(set.resolve(None), &locale("en"));
        assert_eq!(set.resolve(None), set.default_locale());
    }

    #[test]
    fn resolving_an_unsupported_locale_yields_the_default() {
        let set = supported("zh-CN", &["zh-CN", "en"]);

        assert_eq!(set.resolve(Some(&locale("fr-FR"))), &locale("zh-CN"));
    }

    #[test]
    fn resolving_a_supported_locale_yields_that_locale() {
        let set = supported("en", &["en", "zh-CN"]);

        assert_eq!(set.resolve(Some(&locale("zh-CN"))), &locale("zh-CN"));
        assert_eq!(set.resolve(Some(&locale("en"))), &locale("en"));
    }

    #[test]
    fn resolving_never_leaves_the_supported_set() {
        let set = supported("en", &["en", "zh-CN"]);

        for requested in [
            None,
            Some(locale("en")),
            Some(locale("zh-CN")),
            Some(locale("fr-FR")),
        ] {
            assert!(set.contains(set.resolve(requested.as_ref())));
        }
    }
}
