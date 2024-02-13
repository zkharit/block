use crate::constants::BLOCK_ADDRESS_SIZE;

// Account as viewed by the blockchain
#[derive(Clone, Debug)]
pub struct Account {
    // address of the account
    address: [u8; BLOCK_ADDRESS_SIZE],
    // current balance of the account
    balance: u64,
    // the current nonce of the account
    nonce: u64,
    // flag marking if the account is currently a validator for the blockchain
    is_validator: bool,
    // total amount staked for the account
    stake: u64,
}

impl Account {
    pub fn new(address: [u8; BLOCK_ADDRESS_SIZE]) -> Self {
        Self {
            address,
            balance: 0,
            nonce: 0,
            is_validator: false,
            stake: 0
        }
    }

    pub fn get_address(&self) -> [u8; BLOCK_ADDRESS_SIZE] {
        self.address
    }

    pub fn increase_balance(&mut self, amount: u64) {
        self.balance = self.balance + amount
    }

    pub fn decrease_balance(&mut self, amount: u64) {
        self.balance = self.balance - amount
    }

    pub fn get_balance(&self) -> u64 {
        self.balance
    }

    pub fn increase_nonce(&mut self) {
        self.nonce = self.nonce + 1
    }

    pub fn set_nonce(&mut self, nonce: u64) {
        self.nonce = nonce
    }

    pub fn get_nonce(&self) -> u64 {
        self.nonce
    }

    pub fn set_validator(&mut self, is_validator: bool) {
        self.is_validator = is_validator
    }

    pub fn get_validator(&self) -> bool {
        self.is_validator
    }

    pub fn set_stake(&mut self, stake: u64) {
        self.stake = stake
    }

    pub fn get_stake(&self) -> u64 {
        self.stake
    }
}