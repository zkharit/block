// version bytes for prefecing addresses with BLoCK
// also adds a 1 at the end of BLoCK, which isn't necessarily great, but can be used as a kind of visual version number, if later addresses types are generated
// the last character (1) I dont think is guarnteed though, theoretically a large enough number could make this wrap to a 2 (or maybe not, infeasible to test and not super valuable information at this time)
// security of address is not sacrified due to additional size i.e. not subtracting from original address to add in prefix address = prefix (BLoCK1) + normal address size = normal address size + 6 (BLoCK1) 39bits total
pub const ADDRESS_VERSION1_BYTES: &'static [u8; 5] = &[0x03, 0xED, 0x73, 0x45, 0xC0];
// prefix version bytes for exporting private key to WIF format
// much like bitcoin WIF format addresses will be prefixed with K or L if it corresponds to a compressed public key or 5 if an uncompressed public key
pub const WIF_VERSION1_PREFIX_BYTES: &'static [u8; 1] = &[0x80];
// suffix version bytes for signaling that the exported private key in WIF format was used to derive its address from a compressed public key
pub const WIF_VERSION1_COMPRESSED_BYTES: &'static [u8; 1] = &[0x01];
// version bytes used to indicate transaction version
pub const TRANSACTION_VERSION: &'static u8 = &0x01;
// 
pub const ADDRESS_SIZE: usize = 39;
