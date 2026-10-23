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
        for namespace in value.split(',').map(str::trim) {
            if !namespace.is_empty() && !namespaces.iter().any(|known| known == namespace) {
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
