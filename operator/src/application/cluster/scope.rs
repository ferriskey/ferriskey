pub const WATCH_NAMESPACES: &str = "WATCH_NAMESPACES";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchScope {
    Cluster,
    Namespaces(Vec<String>),
}

impl WatchScope {
    pub fn from_env() -> Self {
        Self::parse(std::env::var(WATCH_NAMESPACES).ok().as_deref())
    }

    pub fn parse(value: Option<&str>) -> Self {
        let mut namespaces: Vec<String> = Vec::new();
        for namespace in value.unwrap_or_default().split(',').map(str::trim) {
            if !namespace.is_empty() && !namespaces.iter().any(|known| known == namespace) {
                namespaces.push(namespace.to_string());
            }
        }
        if namespaces.is_empty() {
            Self::Cluster
        } else {
            Self::Namespaces(namespaces)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_value_means_the_whole_cluster() {
        assert_eq!(WatchScope::parse(None), WatchScope::Cluster);
        assert_eq!(WatchScope::parse(Some("")), WatchScope::Cluster);
        assert_eq!(WatchScope::parse(Some(" , ,")), WatchScope::Cluster);
    }

    #[test]
    fn a_comma_separated_list_names_the_watched_namespaces() {
        assert_eq!(
            WatchScope::parse(Some("tenant-a, tenant-b,,tenant-c ")),
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
            WatchScope::parse(Some("a,b,a")),
            WatchScope::Namespaces(vec!["a".to_string(), "b".to_string()])
        );
    }
}
