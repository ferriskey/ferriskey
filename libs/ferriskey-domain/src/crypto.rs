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
pub const OWASP_LOW_MEMORY_ITERATIONS: u32 = 5;
pub const MAX_PASSWORD_HASH_MEMORY_KIB: u32 = 262_144;
pub const MAX_PASSWORD_HASH_ITERATIONS: u32 = 16;
pub const MAX_PASSWORD_HASH_PARALLELISM: u32 = 8;

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
    pub fn owasp_minimum_memory_kib(iterations: u32) -> Option<u32> {
        match iterations {
            0 => None,
            1 => Some(47_104),
            2 => Some(19_456),
            3 => Some(12_288),
            4 => Some(9_216),
            _ => Some(OWASP_MINIMUM_MEMORY_KIB),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.parallelism == 0 {
            return Err("password hashing parallelism must be at least 1".to_string());
        }
        if self.parallelism > MAX_PASSWORD_HASH_PARALLELISM {
            return Err(format!(
                "password hashing parallelism {} exceeds the ceiling of {MAX_PASSWORD_HASH_PARALLELISM}",
                self.parallelism
            ));
        }
        if self.memory_kib > MAX_PASSWORD_HASH_MEMORY_KIB {
            return Err(format!(
                "password hashing memory {} KiB exceeds the ceiling of {MAX_PASSWORD_HASH_MEMORY_KIB} KiB",
                self.memory_kib
            ));
        }
        if self.iterations > MAX_PASSWORD_HASH_ITERATIONS {
            return Err(format!(
                "password hashing iterations {} exceed the ceiling of {MAX_PASSWORD_HASH_ITERATIONS}",
                self.iterations
            ));
        }
        let Some(owasp_floor) = Self::owasp_minimum_memory_kib(self.iterations) else {
            return Err("password hashing iterations must be at least 1".to_string());
        };
        if self.memory_kib < owasp_floor {
            return Err(format!(
                "password hashing with {} iterations needs at least {owasp_floor} KiB to match an OWASP Argon2id configuration, got {} KiB",
                self.iterations, self.memory_kib
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_PASSWORD_HASH_ITERATIONS, MAX_PASSWORD_HASH_MEMORY_KIB, MAX_PASSWORD_HASH_PARALLELISM,
        PasswordHashingCost,
    };

    fn cost(memory_kib: u32, iterations: u32, parallelism: u32) -> PasswordHashingCost {
        PasswordHashingCost {
            memory_kib,
            iterations,
            parallelism,
        }
    }

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
    fn every_owasp_recommended_configuration_passes() {
        for (memory_kib, iterations) in [
            (47_104, 1),
            (19_456, 2),
            (12_288, 3),
            (9_216, 4),
            (7_168, 5),
        ] {
            assert!(
                cost(memory_kib, iterations, 1).validate().is_ok(),
                "{memory_kib} KiB with {iterations} iterations is an OWASP configuration"
            );
        }
        assert!(cost(65_536, 3, 4).validate().is_ok());
    }

    #[test]
    fn costs_below_the_owasp_ladder_are_refused() {
        assert!(cost(4_096, 5, 1).validate().is_err());
        assert!(cost(7_168, 3, 1).validate().is_err());
        assert!(cost(9_216, 3, 1).validate().is_err());
        assert!(cost(19_456, 1, 1).validate().is_err());
        assert!(cost(7_168, 0, 1).validate().is_err());
    }

    #[test]
    fn costs_outside_the_ceilings_shared_with_imports_are_refused() {
        assert!(cost(19_456, 2, 0).validate().is_err());
        assert!(
            cost(65_536, 3, MAX_PASSWORD_HASH_PARALLELISM + 1)
                .validate()
                .is_err()
        );
        assert!(
            cost(65_536, 3, MAX_PASSWORD_HASH_PARALLELISM)
                .validate()
                .is_ok()
        );
        assert!(
            cost(MAX_PASSWORD_HASH_MEMORY_KIB + 1, 5, 1)
                .validate()
                .is_err()
        );
        assert!(cost(MAX_PASSWORD_HASH_MEMORY_KIB, 5, 1).validate().is_ok());
        assert!(
            cost(7_168, MAX_PASSWORD_HASH_ITERATIONS + 1, 1)
                .validate()
                .is_err()
        );
        assert!(
            cost(7_168, MAX_PASSWORD_HASH_ITERATIONS, 1)
                .validate()
                .is_ok()
        );
    }
}
