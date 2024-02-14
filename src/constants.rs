// version bytes for prefecing addresses with BLoCK
// also adds a 1 at the end of BLoCK, which isn't necessarily great, but can be used as a kind of visual version number, if later addresses types are generated
// the last character (1) I dont think is guarnteed though, theoretically a large enough number could make this wrap to a 2 (or maybe not, infeasible to test and not super valuable information at this time)
// security of address is not sacrified due to additional size i.e. not subtracting from original address to add in prefix address = prefix (BLoCK1) + normal address size = normal address size + 6 (BLoCK1) 39bits total
pub const BLOCK_ADDRESS_VERSION1_BYTES: &'static [u8; 5] = &[0x03, 0xED, 0x73, 0x45, 0xC0];
// prefix version bytes for exporting private key to WIF format
// much like bitcoin WIF format addresses will be prefixed with K or L if it corresponds to a compressed public key or 5 if an uncompressed public key
pub const WIF_VERSION1_PREFIX_BYTES: &'static [u8; 1] = &[0x80];
// suffix version bytes for signaling that the exported private key in WIF format was used to derive its address from a compressed public key
pub const WIF_VERSION1_COMPRESSED_BYTES: &'static [u8; 1] = &[0x01];
// version bytes used to indicate transaction version
pub const TRANSACTION_VERSION: &'static u8 = &0x01;
// block version 1 address size in bytes
pub const BLOCK_ADDRESS_SIZE: usize = 39;
// block version 1 wif private key size in bytes
pub const BLOCK_WIF_PRIVATE_KEY_SIZE: usize = 52;
// compressed public key size (32 bytes for x + 1 byte for y even/odd +/-)
pub const COMPRESSED_PUBLIC_KEY_SIZE: usize = 33;
// default config file name
pub const DEFAULT_CONFIG_FILE_NAME: &'static str = "block.conf";
// default config file contents
pub const DEFAULT_CONFIG_OPTIONS_STRING: &'static str = 
r#"[wallet]
wallet_file = "block.wallet"
compressed_public_key = true
wallet_file_version = 1
[validator]
"#;
// version bytes used to indicate block version
pub const BLOCK_VERSION: &'static u32 = &0x01;
// coinbase transaction sender
pub const COINBASE_SENDER: &'static [u8; COMPRESSED_PUBLIC_KEY_SIZE] = &[0x00; COMPRESSED_PUBLIC_KEY_SIZE];
// account that receieves transaction fees for blocks that don't have a proper validator (no coinbase transaction was sent)
pub const LOOSE_CHANGE_RECEVIER: &'static [u8; BLOCK_ADDRESS_SIZE] = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
// validator enable transaction recipient
pub const VALIDATOR_ENABLE_RECIPIENT: &'static [u8; BLOCK_ADDRESS_SIZE] = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01];
// validator revoke transaction recipient
pub const VALIDATOR_REVOKE_RECIPIENT: &'static [u8; BLOCK_ADDRESS_SIZE] = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02];
// serialized genesis block
// ToDo: update this structure
// Block {
//     block_size: 0xF8,
//     block_header: BlockHeader {
//         version: 0x01,
//         prev_hash: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
//         merkle_root: [0xFD, 0xE1, 0x1E, 0x73, 0x72, 0x36, 0xF6, 0x78, 0x00, 0x54, 0xB5, 0x20, 0xD8, 0xB0, 0xF5, 0xA6, 0x50, 0x07, 0xBF, 0xAA, 0xF8, 0x17, 0x3C, 0x42, 0xEB, 0x6C, 0x4F, 0x49, 0x82, 0xE4, 0x4A, 0x52],
//         timestamp: 0x7B
//     }
//     transactions: [
//         Transaction {
//             version: 0x01,
//             amount: 0x12A05F200,
//             fee: 0x00
//             recipient: [0x42, 0x4C, 0x6F, 0x43, 0x4B, 0x31, 0x44, 0x76, 0x76, 0x4E, 0x68, 0x79, 0x4A, 0x78, 0x6F, 0x43, 0x38, 0x34, 0x35, 0x42, 0x45, 0x48, 0x37, 0x44, 0x79, 0x32, 0x53, 0x62, 0x44, 0x48, 0x42, 0x50, 0x70, 0x61, 0x54, 0x77, 0x34, 0x57, 0x38],
//             sender: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
//             signature: [0x7D, 0x3B, 0xF7, 0x40, 0x92, 0x66, 0x67, 0xD8, 0xA2, 0xDD, 0x47, 0x10, 0x06, 0x53, 0x16, 0x41, 0x25, 0x5A, 0xFD, 0x04, 0x32, 0x99, 0xEE, 0x00, 0xF4, 0x34, 0x06, 0x2B, 0x2A, 0x67, 0x4F, 0xE2, 0x69, 0x03, 0xC0, 0xE5, 0x22, 0x5F, 0x71, 0x57, 0x39, 0x1E, 0xCB, 0x09, 0xD3, 0x8F, 0x0F, 0xC1, 0xE5, 0x91, 0x14, 0x65, 0x32, 0xD4, 0x9C, 0x20, 0x5E, 0x1E, 0xB3, 0x81, 0x12, 0x9F, 0x77, 0x21],
//             nonce: 0x00
//         }
//     ]
// }
pub const GENESIS_BLOCK: &'static [u8] = &[0x00, 0x00, 0x00, 0xF8, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFD, 0xE1, 0x1E, 0x73, 0x72, 0x36, 0xF6, 0x78, 0x00, 0x54, 0xB5, 0x20, 0xD8, 0xB0, 0xF5, 0xA6, 0x50, 0x07, 0xBF, 0xAA, 0xF8, 0x17, 0x3C, 0x42, 0xEB, 0x6C, 0x4F, 0x49, 0x82, 0xE4, 0x4A, 0x52, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x01, 0x2A, 0x05, 0xF2, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x42, 0x4C, 0x6F, 0x43, 0x4B, 0x31, 0x44, 0x76, 0x76, 0x4E, 0x68, 0x79, 0x4A, 0x78, 0x6F, 0x43, 0x38, 0x34, 0x35, 0x42, 0x45, 0x48, 0x37, 0x44, 0x79, 0x32, 0x53, 0x62, 0x44, 0x48, 0x42, 0x50, 0x70, 0x61, 0x54, 0x77, 0x34, 0x57, 0x38, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7D, 0x3B, 0xF7, 0x40, 0x92, 0x66, 0x67, 0xD8, 0xA2, 0xDD, 0x47, 0x10, 0x06, 0x53, 0x16, 0x41, 0x25, 0x5A, 0xFD, 0x04, 0x32, 0x99, 0xEE, 0x00, 0xF4, 0x34, 0x06, 0x2B, 0x2A, 0x67, 0x4F, 0xE2, 0x69, 0x03, 0xC0, 0xE5, 0x22, 0x5F, 0x71, 0x57, 0x39, 0x1E, 0xCB, 0x09, 0xD3, 0x8F, 0x0F, 0xC1, 0xE5, 0x91, 0x14, 0x65, 0x32, 0xD4, 0x9C, 0x20, 0x5E, 0x1E, 0xB3, 0x81, 0x12, 0x9F, 0x77, 0x21, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
// halving interval in blocks
pub const HALVING_INTERVAL: &'static u64 = &210000;
// number of the lowest demonination of currency per full coin, this unit is equivalent to satoshis per bitcoin (and the same value)
pub const LOWEST_DENOMINATION_PER_COIN: &'static u64 = &100000000;
// bootstrapping phase minimum block height, used to determine when users need to start staking coins
pub const BOOTSTRAPPING_PHASE_BLOCK_HEIGHT: &'static u64 = &105000;
// minimum amount that needs to be staked to become a validator 32 coins, only used after the bootstrapping phase
pub const MINIMUM_STAKING_AMOUNT: &'static u64 = &3200000000;
