use std::io::Read;
use std::{fs::File, io::Write};
use std::path::{PathBuf, Path};
use std::fs::OpenOptions;
use serde::{Serialize, Deserialize};

use crate::constants::DEFAULT_CONFIG_OPTIONS_STRING;

// ToDo: cehck if the fields of the config should all be public so they can be addressed like: config.wallet.wallet_file, or should there be
// getter functions for them all? I think getters, because if they are public variables then anything can change them and thats not intended
// can add setter functions eventually when in console config editing is supported

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    wallet: WalletConfig,
}

#[derive(Serialize, Deserialize, Debug)]
struct WalletConfig {
    wallet_file: PathBuf
}

impl Config {
    pub fn new(config_file_path: &PathBuf) -> Self {
        // create the config file with the given path if it doesnt exist or open it at the given path if it exists
        let config = match OpenOptions::new().write(true).create_new(true).open(config_file_path) {
            Ok(config_file) => Self::generate_config_file(config_file),
            Err(_) => Self::open_config_file(config_file_path)
        };

        config
    }

    fn generate_config_file(mut config_file: File) -> Self {
        // generate default config, write it to file and return a built Config object
        let config = toml::from_str(DEFAULT_CONFIG_OPTIONS_STRING).unwrap();

        let _ = match config_file.write(toml::to_string(&config).unwrap().as_bytes()) {
            Ok(size) => size,
            Err(..) => panic!("Error writing to config file")
        };

        config
    }

    fn open_config_file(config_file_path: &PathBuf) -> Self {
        // open config file and pass along to read config file function which returns built Config object
        let config = match OpenOptions::new().read(true).open(config_file_path) {
            Ok(config_file) => Self::read_config_file(config_file),
            Err(_) => panic!("Error opening config file at path {}", config_file_path.display())
        };
        
        config
    }

    fn read_config_file(mut config_file: File) -> Self {
        // read contents from config file and parse into a Config object
        let mut config_file_string: String = String::new();
        config_file.read_to_string(&mut config_file_string).unwrap();

        let config: Config = match toml::from_str(&config_file_string) {
            Ok(config) => config,
            Err(_) => panic!("Error parsing config file")
        };

        config
    }

    pub fn get_wallet_file(&self) -> &Path {
        &self.wallet.wallet_file
    }
    
}