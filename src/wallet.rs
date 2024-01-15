use k256::{SecretKey, ecdsa::{SigningKey, VerifyingKey, Signature, signature::Signer}, PublicKey, Secp256k1};
use rand_core::OsRng;
use ripemd::Ripemd160;
use sha2::{Sha256, Digest};

use crate::transaction::Transaction;

pub struct Wallet {
    private_key: elliptic_curve::SecretKey<Secp256k1>,
    public_key: elliptic_curve::PublicKey<Secp256k1>,
    address: Vec<u8>,
    signing_key: k256::ecdsa::SigningKey,
    verifying_key: k256::ecdsa::VerifyingKey,
    wif_private_key: Vec<u8>,
    nonce: u64
}

impl Wallet {
    // version bytes for prefecing addresses with BLoCK
    // also adds a 1 at the end of BLoCK, which isn't necessarily great, but can be used as a kind of visual version number, if later addresses types are generated
    // the last character (1) I dont think is guarnteed though, theoretically a large enough number could make this wrap to a 2 (or maybe not, infeasible to test and not super valuable information at this time)
    // security of address is not sacrified due to additional size i.e. not subtracting from original address to add in prefix address = prefix (BLoCK1) + normal address size = normal address size + 6 (BLoCK1) 39bits total
    const ADDRESS_VERSION1_BYTES: &'static [u8; 5] = &[0x03, 0xED, 0x73, 0x45, 0xC0];
    // prefix version bytes for exporting private key to WIF format
    // much like bitcoin WIF format addresses will be prefixed with K or L if it corresponds to a compressed public key or 5 if an uncompressed public key
    const WIF_VERSION1_PREFIX_BYTES: &'static [u8; 1] = &[0x80];
    // suffix version bytes for signaling that the exported private key in WIF format was used to derive its address from a compressed public key
    const WIF_VERSION1_COMPRESSED_BYTES: &'static [u8; 1] = &[0x01];
    // version bytes used to indicate transaction version
    const TRANSACTION_VERSION: &'static u8 = &0x01;

    pub fn new() -> Self {
        let private_key: elliptic_curve::SecretKey<Secp256k1> = SecretKey::random(&mut OsRng);
        let public_key: elliptic_curve::PublicKey<Secp256k1> = private_key.public_key();
        let address: Vec<u8> = Wallet::generate_address(&public_key);
        let signing_key: SigningKey = SigningKey::from(&private_key);
        let verifying_key: VerifyingKey = VerifyingKey::from(&public_key);
        let wif_private_key: Vec<u8> = Wallet::generate_wif_private_key(&private_key, true);
        let nonce: u64 = 0;
        
        Self {
            private_key,
            public_key,
            address,
            signing_key,
            verifying_key,
            wif_private_key,
            nonce,
        }
    }

    // pub fn from(WalletFile) -> Self {

    // }

    pub fn public_key(&mut self) -> PublicKey {
        self.public_key.clone()
    }

    pub fn address(&mut self) -> Vec<u8> {
        self.address.clone()
    }

    pub fn wif_private_key(&mut self) -> Vec<u8> {
        self.wif_private_key.clone()
    }

    // pub fn save_wallet_file() {

    // }

    // pub fn sign_transaction(&t: Transaction) -> Signature {
    //     Self::sign(t);
    // }

    // ToDo: what other public functions are needed from a wallet?

    fn sign(&self, message: &[u8]) -> Signature {
        return self.signing_key.sign(message)
    }
  
    // ToDo: Look into error handling
    // not sure if tihs should be in wallet, might be needed in "VerificationEngine" or something like that
    // Verification engine would also be used within Wallet, maybe to verify other transactions & blocks
    // What is the use case of this within the wallet? What signatures are you verifying with your own public key?
    // fn verify(&mut self, message: &[u8], signature: Signature) {
    //     return self.verifying_key.verify(message, &signature).unwrap();
    // }

    // Dont think we want a broadcast transaction here, I think should be the responsibility of some BlockNetwork Struct that would take a signed transaction type (or similar)
    // Need to think about the different components, Network, Block/Transaction, Wallet, Validator, and which objects should be responsible for what
    // aka who constructs a Transaction? Wallet (cuz it needs to sign it), or Transaction. Maybe something closer to main can constuct the Transaction with Transaction::new(), but then Network uses the wallet to sign it before it is broadcasted?
    // Idk probably want Wallet and Network separated as much as possible

    fn generate_address(public_key: &elliptic_curve::PublicKey<Secp256k1>) -> Vec<u8> {
        // block addresses are generated in a similar way to version 1 bitcoin addresses
        // the general process can be found here: https://en.bitcoin.it/wiki/Technical_background_of_version_1_Bitcoin_addresses#How_to_create_Bitcoin_Address

        // sha256(compressed public key)
        let mut sha256_hasher: Sha256 = Sha256::new();
        // convert to [u8] array in comprsesed format with 0x02 or 0x03 prefix (if y is even/odd)
        sha256_hasher.update(public_key.to_sec1_bytes());
        let sha256_compressed_pub_key = sha256_hasher.finalize();

        // ripemd160(sha256(compressed public key))
        let mut ripemd160_hasher: Ripemd160 = Ripemd160::new();
        ripemd160_hasher.update(sha256_compressed_pub_key);
        let ripemd_sha256_pub_key = ripemd160_hasher.finalize();

        // version bytes + ripemd160(sha256(compressed public key))
        let mut vec_of_ripe_sha_pub_key = ripemd_sha256_pub_key.to_vec();
        // push the version bytes to the end
        Self::ADDRESS_VERSION1_BYTES.iter().for_each(|item| {
            vec_of_ripe_sha_pub_key.push(*item);
        });
        // then rotate the vector so the version bytes are at the front
        vec_of_ripe_sha_pub_key.rotate_right(Self::ADDRESS_VERSION1_BYTES.len());

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

        // base58 encode version bytes + ripemd160(sha256(compressed public key)) + first 4 bytes of checksum
        bs58::encode(vec_of_ripe_sha_pub_key).into_vec()
    }

    fn generate_wif_private_key(private_key: &elliptic_curve::SecretKey<Secp256k1>, compressed: bool) -> Vec<u8> {
        // private keys are encoded in WIF format similar to bitcoin's WIF format
        // the general process can be found here: https://en.bitcoin.it/wiki/Wallet_import_format#Private_key_to_WIF

        // WIF version bytes + private key
        let mut private_key_vec = private_key.to_bytes().to_vec();
        // push the version bytes to the front
        Self::WIF_VERSION1_PREFIX_BYTES.iter().for_each(|item| {
            private_key_vec.push(*item);
        });
        // then rotate the vector so the version bytes are at the front
        private_key_vec.rotate_right(Self::WIF_VERSION1_PREFIX_BYTES.len());

        // if the private key is used to derive addresses from a compressed public key append the suffix bytes to the end
        if compressed {
            Self::WIF_VERSION1_COMPRESSED_BYTES.iter().for_each(|item| {
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
        bs58::encode(private_key_vec).into_vec()
    }
}
