#[derive(Debug, Clone)]
pub struct HashResult {
    pub hash: String,
    pub salt: String,
    pub hash_iterations: u32,
    pub algorithm: String,
}

impl HashResult {
    pub fn new(hash: String, salt: String, hash_iterations: u32, algorithm: String) -> Self {
        Self {
            hash,
            salt,
            hash_iterations,
            algorithm,
        }
    }
}

pub const OWASP_MINIMUM_MEMORY_KIB: u32 = 7_168;
pub const OWASP_MINIMUM_ITERATIONS: u32 = 2;
pub const OWASP_LOW_MEMORY_ITERATIONS: u32 = 5;
pub const OWASP_HIGH_MEMORY_KIB: u32 = 19_456;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PasswordHashingCost {
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

impl Default for PasswordHashingCost {
    fn default() -> Self {
        Self {
            memory_kib: OWASP_MINIMUM_MEMORY_KIB,
            iterations: OWASP_LOW_MEMORY_ITERATIONS,
            parallelism: 1,
        }
    }
}

impl PasswordHashingCost {
    pub fn validate(&self) -> Result<(), String> {
        if self.parallelism == 0 {
            return Err("password hashing parallelism must be at least 1".to_string());
        }
        if self.memory_kib < OWASP_MINIMUM_MEMORY_KIB {
            return Err(format!(
                "password hashing memory {} KiB is below the OWASP floor of {OWASP_MINIMUM_MEMORY_KIB} KiB",
                self.memory_kib
            ));
        }
        if self.iterations < OWASP_MINIMUM_ITERATIONS {
            return Err(format!(
                "password hashing iterations {} are below the OWASP floor of {OWASP_MINIMUM_ITERATIONS}",
                self.iterations
            ));
        }
        if self.memory_kib < OWASP_HIGH_MEMORY_KIB && self.iterations < OWASP_LOW_MEMORY_ITERATIONS
        {
            return Err(format!(
                "password hashing with {} KiB needs at least {OWASP_LOW_MEMORY_ITERATIONS} iterations, or {OWASP_HIGH_MEMORY_KIB} KiB with {OWASP_MINIMUM_ITERATIONS}",
                self.memory_kib
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::PasswordHashingCost;

    #[test]
    fn the_default_cost_is_an_owasp_recommended_configuration() {
        let cost = PasswordHashingCost::default();
        assert_eq!(
            (cost.memory_kib, cost.iterations, cost.parallelism),
            (7_168, 5, 1)
        );
        assert!(cost.validate().is_ok());
    }

    #[test]
    fn the_high_memory_low_iteration_configuration_passes() {
        let cost = PasswordHashingCost {
            memory_kib: 19_456,
            iterations: 2,
            parallelism: 1,
        };
        assert!(cost.validate().is_ok());
    }

    #[test]
    fn costs_below_the_floor_are_refused() {
        let weak_memory = PasswordHashingCost {
            memory_kib: 4_096,
            iterations: 5,
            parallelism: 1,
        };
        let weak_iterations = PasswordHashingCost {
            memory_kib: 7_168,
            iterations: 3,
            parallelism: 1,
        };
        let no_parallelism = PasswordHashingCost {
            memory_kib: 19_456,
            iterations: 2,
            parallelism: 0,
        };
        assert!(weak_memory.validate().is_err());
        assert!(weak_iterations.validate().is_err());
        assert!(no_parallelism.validate().is_err());
    }
}
