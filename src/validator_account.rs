use crate::constants::COMPRESSED_PUBLIC_KEY_SIZE;

#[derive(Debug, Clone)]
pub struct ValidatorAccount {
    // public key of the validator
    public_key: [u8; COMPRESSED_PUBLIC_KEY_SIZE],
}

impl ValidatorAccount {
    pub fn new(public_key: [u8; COMPRESSED_PUBLIC_KEY_SIZE]) -> Self {
        Self {
            public_key,
        }
    }

    pub fn get_public_key(&self) -> [u8; COMPRESSED_PUBLIC_KEY_SIZE] {
        self.public_key
    }
}