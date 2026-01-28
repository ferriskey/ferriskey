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
