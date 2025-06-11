#[macro_export]
macro_rules! btreemap {
    ($($k:expr => $v:expr),* $(,)?) => {{
        let mut map = ::std::collections::BTreeMap::new();
        $( map.insert($k, $v); )*
        map
    }};
}
