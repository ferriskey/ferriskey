use std::env::VarError;

use crate::domain::error::OperatorError;

pub const WATCH_NAMESPACES: &str = "WATCH_NAMESPACES";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchScope {
    Cluster,
    Namespaces(Vec<String>),
}

impl WatchScope {
    pub fn from_env() -> Result<Self, OperatorError> {
        match std::env::var(WATCH_NAMESPACES) {
            Ok(value) => Self::parse(Some(&value)),
            Err(VarError::NotPresent) => Self::parse(None),
            Err(VarError::NotUnicode(_)) => Err(OperatorError::InvalidConfiguration {
                message: format!("{WATCH_NAMESPACES} is not valid unicode"),
            }),
        }
    }

    pub fn parse(value: Option<&str>) -> Result<Self, OperatorError> {
        let value = value.unwrap_or_default();
        let mut namespaces: Vec<String> = Vec::new();
        for namespace in value
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            if !is_dns_label(namespace) {
                return Err(OperatorError::InvalidConfiguration {
                    message: format!(
                        "{WATCH_NAMESPACES} lists {namespace:?}, which is not a valid namespace name"
                    ),
                });
            }
            if !namespaces.iter().any(|known| known == namespace) {
                namespaces.push(namespace.to_string());
            }
        }
        if !namespaces.is_empty() {
            Ok(Self::Namespaces(namespaces))
        } else if value.is_empty() {
            Ok(Self::Cluster)
        } else {
            Err(OperatorError::InvalidConfiguration {
                message: format!(
                    "{WATCH_NAMESPACES} is {value:?} and lists no namespace; leave it unset to watch the whole cluster"
                ),
            })
        }
    }
}

fn is_dns_label(name: &str) -> bool {
    let bytes = name.as_bytes();
    let is_edge = |byte: &u8| byte.is_ascii_lowercase() || byte.is_ascii_digit();
    !bytes.is_empty()
        && bytes.len() <= 63
        && bytes.first().is_some_and(is_edge)
        && bytes.last().is_some_and(is_edge)
        && bytes.iter().all(|byte| is_edge(byte) || *byte == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(value: Option<&str>) -> WatchScope {
        WatchScope::parse(value).expect("a valid setting")
    }

    #[test]
    fn no_value_means_the_whole_cluster() {
        assert_eq!(parsed(None), WatchScope::Cluster);
        assert_eq!(parsed(Some("")), WatchScope::Cluster);
    }

    #[test]
    fn a_setting_that_lists_no_namespace_is_refused() {
        for value in [",", " , ", " , ,", " ", "\n"] {
            assert!(
                matches!(
                    WatchScope::parse(Some(value)),
                    Err(OperatorError::InvalidConfiguration { .. })
                ),
                "{value:?} must not widen the scope to the whole cluster"
            );
        }
    }

    #[test]
    fn a_name_that_is_not_a_dns_label_is_refused() {
        let too_long = "a".repeat(64);
        for value in [
            "Team_A",
            "team-a,Team_B",
            "-a",
            "a-",
            "a.b",
            "a b",
            "équipe",
            too_long.as_str(),
        ] {
            assert!(
                matches!(
                    WatchScope::parse(Some(value)),
                    Err(OperatorError::InvalidConfiguration { .. })
                ),
                "{value:?} must be refused"
            );
        }
    }

    #[test]
    fn a_valid_dns_label_is_accepted() {
        let longest = "a".repeat(63);
        for value in ["a", "0a", "a-1", "tenant-a", longest.as_str()] {
            assert_eq!(
                parsed(Some(value)),
                WatchScope::Namespaces(vec![value.to_string()])
            );
        }
    }

    #[test]
    fn a_comma_separated_list_names_the_watched_namespaces() {
        assert_eq!(
            parsed(Some("tenant-a, tenant-b,,tenant-c ")),
            WatchScope::Namespaces(vec![
                "tenant-a".to_string(),
                "tenant-b".to_string(),
                "tenant-c".to_string(),
            ])
        );
    }

    #[test]
    fn a_namespace_listed_twice_is_watched_once() {
        assert_eq!(
            parsed(Some("a,b,a")),
            WatchScope::Namespaces(vec!["a".to_string(), "b".to_string()])
        );
    }
}
