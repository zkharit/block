use elliptic_curve::{SecretKey, sec1::ToEncodedPoint};
use k256::{ecdsa::{SigningKey, Signature, signature::Signer}, PublicKey, Secp256k1};
use rand_core::OsRng;
use ripemd::Ripemd160;
use sha2::{Sha256, Digest};

use std::{ fs::File, io::{self, Write}, path::Path};

use crate::config::WalletConfig;
use crate::constants::{BLOCK_ADDRESS_VERSION1_BYTES, WIF_VERSION1_PREFIX_BYTES, WIF_VERSION1_COMPRESSED_BYTES, TRANSACTION_VERSION, BLOCK_ADDRESS_SIZE, COMPRESSED_PUBLIC_KEY_SIZE};
use crate::transaction::{Transaction, TxMetadata};
use crate::util::{open_file_read, create_file_new, read_file_from_beginning, open_file_write};

pub struct Wallet {
    public_key: elliptic_curve::PublicKey<Secp256k1>,
    address: [u8; BLOCK_ADDRESS_SIZE],
    nonce: u64,
    config: WalletConfig
}

impl Wallet {
    pub fn new(config: WalletConfig) -> Self {

        // get wallet file path from the config
        let wallet_file_path = config.get_wallet_file();

        // attempt to open wallet file, create it if it doesnt exist, or exit if other error
        let wallet_file = match open_file_read(wallet_file_path) {
            Ok(wallet_file) => wallet_file,
            Err(error) => match error.kind() {
                io::ErrorKind::NotFound => match  Self::generate_wallet_file(wallet_file_path, config.get_compressed_public_key(), config.get_wallet_file_version()) {
                    Ok(wallet_file) => wallet_file,
                    Err(_) => panic!("Error creating new wallet file")
                },
                _ => panic!("Error creating wallet file")
            }
        };

        // read private key and nonce from wallet key file
        let (private_key, nonce) = match Self::read_wallet_file(wallet_file, config.get_compressed_public_key()) {
            Some(wallet_file_tuple) => wallet_file_tuple,
            None => {
                panic!("Error reading wallet file");
            }
        };

        // obtain public key and address
        let public_key: elliptic_curve::PublicKey<Secp256k1> = private_key.public_key();
        let address: [u8; BLOCK_ADDRESS_SIZE] = Wallet::generate_address(&public_key, config.get_compressed_public_key());
        
        Self {
            public_key,
            address,
            nonce,
            config,
        }
    }

    pub fn get_public_key(&self) -> PublicKey {
        self.public_key
    }

    pub fn get_address(&self) -> [u8; BLOCK_ADDRESS_SIZE] {
        self.address
    }

    pub fn get_nonce(&self) -> u64 {
        self.nonce
    }

    pub fn set_nonce(&mut self, n: u64) {
        self.nonce = n;
        // update nonce in wallet file (also re-writes private key)
        self.update_wallet_file();
    }

    pub fn create_tx(&mut self, amount: u64, fee: u64, recipient: [u8; BLOCK_ADDRESS_SIZE]) -> Option<Transaction> {
        // obtain the wallet signing key from the wallet file
        let signing_key = match self.get_signing_key() {
            Some(signing_key) => signing_key,
            None => {
                println!("Failed to create transaction, could not obtain signing key");
                return None
            }
        };

        // get the signature for the transaction
        let tx_sig = Self::create_tx_sig(signing_key, *TRANSACTION_VERSION, amount, fee, recipient, self.nonce);

        // convert vector into [u8; COMPRESSED_PUBLIC_KEY_SIZE]
        let sender_pub_key_vec = self.get_public_key().to_sec1_bytes().to_vec();
        let sender_pub_key: [u8; COMPRESSED_PUBLIC_KEY_SIZE] = sender_pub_key_vec.try_into().unwrap();
        
        // create the transaction
        let tx = Transaction::new(*TRANSACTION_VERSION, amount, fee, recipient, sender_pub_key, tx_sig, self.nonce);

        // increment the wallet nonce after the transaction was created
        self.set_nonce(self.nonce + 1);

        Some(tx)
    }

    fn create_tx_sig(signing_key: SigningKey, version: u8, amount: u64, fee: u64, recipient: [u8; BLOCK_ADDRESS_SIZE], nonce: u64) -> Signature {
        // serialize the transaction metadata
        let hashed_serialized_tx_metadata = TxMetadata::serialize_hash_tx_metadata(&TxMetadata::new(version, amount, fee, recipient, nonce));

        Self::sign(signing_key, &hashed_serialized_tx_metadata)
    }

    fn sign(signing_key: SigningKey, message: &[u8]) -> Signature {
        signing_key.sign(message)
    }

    fn generate_address(public_key: &elliptic_curve::PublicKey<Secp256k1>, compressed: bool) -> [u8; BLOCK_ADDRESS_SIZE] {
        // block addresses are generated in a similar way to version 1 bitcoin addresses
        // the general process can be found here: https://en.bitcoin.it/wiki/Technical_background_of_version_1_Bitcoin_addresses#How_to_create_Bitcoin_Address

        // sha256(compressed public key)
        let mut sha256_hasher: Sha256 = Sha256::new();
        // convert to [u8] array in comprsesed format with 0x02 or 0x03 prefix (if y is even/odd) or not compressed
        sha256_hasher.update(public_key.to_encoded_point(compressed));
        let sha256_pub_key = sha256_hasher.finalize();

        // ripemd160(sha256(compressed public key))
        let mut ripemd160_hasher: Ripemd160 = Ripemd160::new();
        ripemd160_hasher.update(sha256_pub_key);
        let ripemd_sha256_pub_key = ripemd160_hasher.finalize();

        // version bytes + ripemd160(sha256(compressed public key))
        let mut vec_of_ripe_sha_pub_key = ripemd_sha256_pub_key.to_vec();
        // push the version bytes to the end
        BLOCK_ADDRESS_VERSION1_BYTES.iter().for_each(|item| {
            vec_of_ripe_sha_pub_key.push(*item);
        });
        // then rotate the vector so the version bytes are at the front
        vec_of_ripe_sha_pub_key.rotate_right(BLOCK_ADDRESS_VERSION1_BYTES.len());

        // sha256(sha256(version bytes + ripemd160(sha256(compressed public key)))) to get checksum 
        let mut sha256_hasher: Sha256 = Sha256::new();
        sha256_hasher.update(&vec_of_ripe_sha_pub_key);
        let first_sha256 = sha256_hasher.finalize();

        let mut sha256_hasher: Sha256 = Sha256::new();
        sha256_hasher.update(first_sha256);
        let second_sha256 = sha256_hasher.finalize();

        // version bytes + ripemd160(sha256(compressed public key)) + first 4 bytes of checksum
        // push the first 4 bytes of the second sha as the checksum at the end of the version bytes + ripe(sha(pub))
        vec_of_ripe_sha_pub_key.push(second_sha256[0]);
        vec_of_ripe_sha_pub_key.push(second_sha256[1]);
        vec_of_ripe_sha_pub_key.push(second_sha256[2]);
        vec_of_ripe_sha_pub_key.push(second_sha256[3]);

        // base58 encode version bytes + ripemd160(sha256(compressed public key)) + first 4 bytes of 
        let address: Vec<u8> = bs58::encode(vec_of_ripe_sha_pub_key).into_vec();
        // convert vector into [u8; BLOCK_ADDRESS_SIZE]
        address.clone().try_into().unwrap()
    }

    fn generate_wif_private_key(private_key: &elliptic_curve::SecretKey<Secp256k1>, compressed: bool) -> Vec<u8> {
        // private keys are encoded in WIF format similar to bitcoin's WIF format
        // the general process can be found here: https://en.bitcoin.it/wiki/Wallet_import_format#Private_key_to_WIF

        // WIF version bytes + private key
        let mut private_key_vec = private_key.to_bytes().to_vec();
        // push the version bytes to the front
        WIF_VERSION1_PREFIX_BYTES.iter().for_each(|item| {
            private_key_vec.push(*item);
        });
        // then rotate the vector so the version bytes are at the front
        private_key_vec.rotate_right(WIF_VERSION1_PREFIX_BYTES.len());

        // if the private key is used to derive addresses from a compressed public key append the suffix bytes to the end
        if compressed {
            WIF_VERSION1_COMPRESSED_BYTES.iter().for_each(|item| {
                private_key_vec.push(*item);
            });
        }

        // sha256(sha256(version bytes + ripemd160(sha256(compressed public key)))) to get checksum 
        let mut sha256_hasher: Sha256 = Sha256::new();
        sha256_hasher.update(&private_key_vec);
        let first_sha256 = sha256_hasher.finalize();

        let mut sha256_hasher: Sha256 = Sha256::new();
        sha256_hasher.update(first_sha256);
        let second_sha256 = sha256_hasher.finalize();

        // version bytes + ripemd160(sha256(compressed public key)) + first 4 bytes of checksum
        // push the first 4 bytes of the second sha as the checksum at the end of the version bytes + ripe(sha(pub))
        private_key_vec.push(second_sha256[0]);
        private_key_vec.push(second_sha256[1]);
        private_key_vec.push(second_sha256[2]);
        private_key_vec.push(second_sha256[3]);

        // base58 encode version bytes + ripemd160(sha256(compressed public key)) + first 4 bytes of checksum
        let wif_private_key: Vec<u8> = bs58::encode(private_key_vec).into_vec();

        wif_private_key
        
    }

    fn wif_to_private_key(wif_private_key_string: &str, compressed: bool) -> Option<elliptic_curve::SecretKey<Secp256k1>> {
        // decode private key string from base58 encoding to bytes
        let mut decoded_private_key = match bs58::decode(wif_private_key_string).into_vec() {
            Ok(decoded_private_key) => decoded_private_key,
            Err(_) => {
                println!("Error decoding private key string from wallet file");
                return None
            }
        };

        // remove the checksum bytes from the decoded private key
        let mut decoded_private_key_checksum: Vec<u8> = vec![];
        decoded_private_key_checksum.push(decoded_private_key.pop()?);
        decoded_private_key_checksum.push(decoded_private_key.pop()?);
        decoded_private_key_checksum.push(decoded_private_key.pop()?);
        decoded_private_key_checksum.push(decoded_private_key.pop()?);

        // ToDo: need to check the checksum?
        // reverse the checksum vector so it is in the correct order
        decoded_private_key_checksum.reverse();

        // check the verion byte
        if decoded_private_key[0].eq(&WIF_VERSION1_PREFIX_BYTES[0]) {
            // remove the version byte(s)
            let mut i = 0;
            while i < WIF_VERSION1_PREFIX_BYTES.len() {
                decoded_private_key.remove(i);
                i += 1;
            }

            // if address is from compressed public key remove the compressed public key byte
            if compressed {
                // pop the compressed public key byte
                decoded_private_key.pop();
            }
        } else {
            // no other private key version bytes have been implemented as of yet
            println!("Invalid private key version bytes found");
            return None
        }

        let private_key = match SecretKey::from_bytes(decoded_private_key.as_slice().into()) {
            Ok(private_key) => private_key,
            Err(_) => panic!("Unable to create private key from wallet file")
        };

        Some(private_key)
    }

    fn get_signing_key(&self) -> Option<SigningKey> {
        // open the wallet file in read mode
        let wallet_file = match Self::open_wallet_file_read(self.config.get_wallet_file()) {
            Some(wallet_file) => wallet_file,
            None => {
                println!("Failed to open wallet file");
                return None
            }
        };

        // obtain the private key fro mthe wallet file
        let private_key = match Self::read_wallet_file(wallet_file, self.config.get_compressed_public_key()) {
            Some((private_key, _)) => private_key,
            None => {
                println!("Failed to obtain signing key from wallet file");
                return None
            }
        };

        // convert the private key into a signing key
        Some(SigningKey::from(private_key))
    }

    fn get_private_key(&self) -> Option<SecretKey<Secp256k1>> {
        // open wallet file in read mode
        let wallet_file = match Self::open_wallet_file_read(self.config.get_wallet_file()) {
            Some(wallet_file) => wallet_file,
            None => {
                println!("Failed to open wallet file");
                return None
            }
        };

        // obtain the private key from the wallet file
        let private_key = match Self::read_wallet_file(wallet_file, self.config.get_compressed_public_key()) {
            Some((private_key, _)) => private_key,
            None => {
                println!("Failed to obtain signing key from wallet file");
                return None
            }
        };

        Some(private_key)
    }

    fn generate_wallet_file(wallet_file_path: &Path, compressed: bool, wallet_file_version: u64) -> Result<File, io::Error> {
        // create new wallet file and fail if it already exists
        let wallet_file = match create_file_new(wallet_file_path) {
            Ok(wallet_file) => wallet_file,
            Err(_) => panic!("Error creating wallet file, it already exists")
        };

        // generate initial private key and nonce
        let private_key: elliptic_curve::SecretKey<Secp256k1> = SecretKey::random(&mut OsRng);
        let nonce: u64 = 0;

        // write private key and nonce to wallet file
        Self::write_to_wallet_file(&wallet_file, &private_key, nonce, compressed, wallet_file_version)?;

        Ok(wallet_file)
    }

    fn open_wallet_file_read(wallet_file_path: &Path) -> Option<File> {
        let wallet_file = match open_file_read(wallet_file_path) {
            Ok(wallet_file) => wallet_file,
            Err(_) => {
                println!("Error opening wallet file to read");
                return None
            }
        };

        Some(wallet_file)
    }

    fn open_wallet_file_write(wallet_file_path: &Path) -> Option<File>{
        let wallet_file = match open_file_write(wallet_file_path) {
            Ok(wallet_file) => wallet_file,
            Err(_) => {
                println!("Error opening wallet file to write");
                return None
            }
        };

        Some(wallet_file)
    }
    
    fn read_wallet_file(wallet_file: File, compressed: bool) -> Option<(SecretKey<Secp256k1>, u64)> {
        // read wallet file and return a tuple of the private key and the nonce
        // if nonce cannot be obtained it will default to 0

        // read wallet file
        let wallet_file_string = match read_file_from_beginning(wallet_file) {
            Ok(wallet_file_string) => wallet_file_string,
            Err(_) => {
                println!("Error reading wallet file");
                return None
            }
        };

        // create wallet object from string
        let wallet_file_string_parts: Vec<&str> = wallet_file_string.split('\n').collect();

        // get wallet file version
        let wallet_file_version = match wallet_file_string_parts.get(0) {
            Some(wallet_file_version) => wallet_file_version,
            None => {
                println!("Error obtaining wallet file version");
                return None
            }
        };

        // parse wallet file version
        let version: u64 = match wallet_file_version.parse::<u64>() {
            Ok(version) => version,
            Err(_) => {
                println!("Error parsing wallet file version");
                return None
            }
        };

        let private_key: SecretKey<Secp256k1>;
        let nonce: u64;

        if version == 1 {
            // get private key string from wallet file fail else
            let wif_private_key_string = match wallet_file_string_parts.get(1) {
                Some(wif_private_key_string) => wif_private_key_string,
                None => {
                    println!("Error obtaining private key from wallet file");
                    return None
                }
            };

            // get nonce from wallet file, use 0 else
            let wallet_nonce = match wallet_file_string_parts.get(2) {
                Some(wallet_nonce) => wallet_nonce,
                None => {
                    println!("Unable to obtain account nonce, defaulting to 0");
                    "0"
                }
            };

            // generate private key from wif string
            private_key = match Self::wif_to_private_key(wif_private_key_string, compressed) {
                Some(private_key) => private_key,
                None => {
                    println!("Unable to convert WIF to private key");
                    return None
                }
            };

            // parse nonce from obtained nonce value 
            nonce = match wallet_nonce.parse::<u64>() {
                Ok(nonce) => nonce,
                Err(_) => {
                    println!("Unable to parse account nonce, defaulting to 0");
                    0
                }
            };
        } else {
            // if unknown wallet file versoin obtained
            println!("Unknown wallet file version obtained");
            return None
        }

        Some((private_key, nonce))

    }

    fn write_to_wallet_file(mut wallet_file: &File, private_key: &elliptic_curve::SecretKey<Secp256k1>, nonce: u64, compressed: bool, wallet_file_version: u64) -> Result<(), io::Error> {
        // create wallet file buffer
        let mut wallet_file_buffer:Vec<u8> = vec![];

        // generate Wallet Import Format (wif) private key
        let wif_private_key = Self::generate_wif_private_key(&private_key, compressed);

        // append wallet_file_version, wif private key and nonce to wallet file
        wallet_file_buffer.extend_from_slice(wallet_file_version.to_string().as_bytes());
        wallet_file_buffer.push(b'\n');
        wallet_file_buffer.extend_from_slice(&wif_private_key);
        wallet_file_buffer.push(b'\n');
        wallet_file_buffer.extend_from_slice(&nonce.to_string().as_bytes());

        // write to wallet file
        let _ = match wallet_file.write(&wallet_file_buffer) {
            Ok(_) => (),
            Err(_) => panic!("Error writing to wallet file")
        };

        Ok(())
    }

    fn update_wallet_file(&mut self) -> Option<()>{
        // obtain the wallet private key from the wallet file
        let private_key = match self.get_private_key() {
            Some(signing_key) => signing_key,
            None => {
                println!("Failed to update wallet file, could not obtain private key");
                return None
            }
        };

        // open wallet file in write mode
        let wallet_file = match Self::open_wallet_file_write(self.config.get_wallet_file()) {
            Some(wallet_file) => wallet_file,
            None => {
                println!("Failed to update wallet file, could not write to wallet file");
                return None
            }
        };

        // write private key and nonce to wallet file
        let _ = match Self::write_to_wallet_file(&wallet_file, &private_key, self.get_nonce(), self.config.get_compressed_public_key(), self.config.get_wallet_file_version()) {
            Ok(_) => (),
            Err(_) => {
                println!("Failed to update wallet file");
                return None
            }
        };

        Some(())
    }

    // Dont think we want a broadcast transaction here, I think should be the responsibility of some BlockNetwork Struct that would take a signed transaction type (or similar)
    // Need to think about the different components, Network, Block/Transaction, Wallet, Validator, and which objects should be responsible for what
    // aka who constructs a Transaction? Wallet (cuz it needs to sign it), or Transaction. Maybe something closer to main can constuct the Transaction with Transaction::new(), but then Network uses the wallet to sign it before it is broadcasted?
    // Idk probably want Wallet and Network separated as much as possible
}
