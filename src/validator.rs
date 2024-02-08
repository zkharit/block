use crate::config::ValidatorConfig;
use crate::transaction::Transaction;
use crate::wallet::Wallet;

pub struct Validator {
    // config for the validator
    config: ValidatorConfig,
    // wallet to be used for validation purposes
    wallet: Wallet
}

impl Validator {
    pub fn new(config: ValidatorConfig, wallet: Wallet) -> Self {
        Self {
            config,
            wallet,
        }
    }

    pub fn create_coinbase_tx(&mut self) -> Option<Transaction>{
        // ToDo: Since validator blocks will be proven to be propsed by the validator by signing the block header, there is nothing stopping a malicious node from changing the included coinbase transaction to whatever (send to WHOever) they want
        // Actually unsure about this, because they merkle root within the block header will be calculated with the coinbase transaction, and the coinbase transaction contains the recipient selected by the validator
        // So this may be ok, Need to think about this tomorrow, but Im pretty sure current "unauthenticated" coinbase tx is fine
        let coinbase_tx = match self.wallet.create_coinbase_tx(50, self.wallet.get_address()) {
            Some(coinbase_tx) => coinbase_tx,
            None => {
                println!("Failed to create coinbase tx");
                return None
            }
        };

        Some(coinbase_tx)
    }
}

