error: very complex type used. Consider factoring parts into `type` definitions
Error:    --> core/src/application/services.rs:134:30
    |
134 |       pub(crate) auth_service: AuthServiceImpl<
    |  ______________________________^
135 | |         RealmRepo,
136 | |         ClientRepo,
137 | |         RedirectUriRepo,
...   |
147 | |         FederationRepo,
148 | |     >,
    | |_____^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.93.0/index.html#type_complexity
    = note: `-D clippy::type-complexity` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::type_complexity)]`
