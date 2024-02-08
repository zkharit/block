use std::fs::File;
use std::{io, io::Write};
use std::path::{PathBuf, Path};
use serde::{Serialize, Deserialize};

use crate::constants::DEFAULT_CONFIG_OPTIONS_STRING;

use crate::util::{create_file_new, open_file_read, read_file_from_beginning};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    wallet: WalletConfig,
    validator: ValidatorConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WalletConfig {
    // path of the wallet file
    wallet_file: PathBuf,
    // if the wallet address was derived from a compressed public key
    compressed_public_key: bool,
    // wallet file version
    wallet_file_version: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValidatorConfig {

}

impl Config {
    pub fn new(config_file_path: &Path) -> Self {
        // attempt to open config file, create it if it doesnt exist, or exit if other error
        let config_file = match open_file_read(config_file_path) {
            Ok(config_file) => config_file,
            Err(error) => match error.kind() {
                io::ErrorKind::NotFound => match Self::generate_config_file(config_file_path) {
                    Ok(config_file) => config_file,
                    Err(_) => panic!("Error creating new config file")
                },
                _ => panic!("Error creating config file")
            }
        };

        // read config file
        let config_file_string = match read_file_from_beginning(config_file) {
            Ok(config_file_string) => config_file_string,
            Err(_) => panic!("Error reading config file")
        };

        // create config object from config string
        let config: Config = match toml::from_str(&config_file_string) {
            Ok(config) => config,
            Err(_) => panic!("Error parsing config file")
        };

        config
    }

    fn generate_config_file(config_file_path: &Path) -> Result<File, io::Error> {
        // create new config file and fail if it already exists
        let mut config_file = match create_file_new(config_file_path) {
            Ok(config_file) => config_file,
            Err(_) => panic!("Error creating new config file")
        };

        // write default config string to the config file
        let _ = match config_file.write(DEFAULT_CONFIG_OPTIONS_STRING.as_bytes()) {
            Ok(_) => (),
            Err(_) => panic!("Error writing initial config file")
        };

        Ok(config_file)
    }

    pub fn get_wallet_config(&self) -> WalletConfig {
        self.wallet.clone()
    }

    pub fn get_validator_config(&self) -> ValidatorConfig {
        self.validator.clone()
    }
    
}

impl WalletConfig {
    pub fn get_wallet_file(&self) -> &Path {
        &self.wallet_file
    }

    pub fn get_compressed_public_key(&self) -> bool {
        self.compressed_public_key
    }

    pub fn get_wallet_file_version(&self) -> u64 {
        self.wallet_file_version
    }
}

impl ValidatorConfig {

}