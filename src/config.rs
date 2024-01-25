use std::fs::File;
use std::{io, io::Write};
use std::path::{PathBuf, Path};
use serde::{Serialize, Deserialize};

use crate::constants::DEFAULT_CONFIG_OPTIONS_STRING;

use crate::util::{create_file_new, open_file_read, read_file_from_beginning};

// ToDo: cehck if the fields of the config should all be public so they can be addressed like: config.wallet.wallet_file, or should there be
// getter functions for them all? I think getters, because if they are public variables then anything can change them and thats not intended
// can add setter functions eventually when in console config editing is supported

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub wallet: WalletConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WalletConfig {
    wallet_file: PathBuf
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

    pub fn get_wallet_config(&self) -> &WalletConfig {
        &self.wallet
    }
    
}

impl WalletConfig {
    pub fn get_wallet_file(&self) -> &Path {
        &self.wallet_file
    }
}