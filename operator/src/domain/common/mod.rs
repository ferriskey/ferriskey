pub struct OperatorConfig {
    pub env: Environment,
}

pub enum Environment {
    Test,
    Development,
    Production,
}
