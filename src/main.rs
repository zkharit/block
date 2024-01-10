use k256::{SecretKey, ecdsa::{SigningKey, VerifyingKey, Signature, signature::Signer, signature::Verifier}};
use rand_core::OsRng;
use ripemd::Ripemd160;
use sha2::{Sha256, Digest};

fn main() {
    // Test secret key found on: https://en.bitcoin.it/wiki/Technical_background_of_version_1_Bitcoin_addresses#How_to_create_Bitcoin_Address
    // Import key, as well as used for static key testing
    // let secret_key_material: &[u8] = &[24, 225, 74, 123, 106, 48, 127, 66, 106, 148, 248, 17, 71, 1, 231, 200, 231, 116, 231, 249, 164, 126, 44, 32, 53, 219, 41, 162, 6, 50, 23, 37];
    // let secret_key_material: &[u8] = &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];

    // This is 2**256 - 2**32 - 2**9 - 2**8 - 2**7 - 2**6 - 2**4 - 1 which I believe is supposed to be the max number in ecdsa, but this breaks when attempting to generate a key from it (or minus 1)
    // let secret_key_material: &[u8] = &[255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 254, 255, 255, 252, 47];

    // used in bitcoin-core/secp256k1 ecdsa_impl.h for some reason? which also breaks when attempting to generate a key from it, BUT 1 less than it seems to work, so that must be the max value for sec256kp1?
    // let secret_key_material: &[u8] = &[255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 254, 186, 174, 220, 230, 175, 72, 160, 59, 191, 210, 94, 140, 208, 54, 65, 65];
    // let secret_key_material: &[u8] = &[255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 254, 186, 174, 220, 230, 175, 72, 160, 59, 191, 210, 94, 140, 208, 54, 65, 64];
    
    // let secret_key = SecretKey::from_bytes(secret_key_material.into()).unwrap();

    // generate private key
    let secret_key = SecretKey::random(&mut OsRng);
    // add the .to_vec here so we can manipulate it later (for private key storage/encoding to wallet import format (WIF))
    let secret_key_bytes = secret_key.to_bytes().to_vec(); // remove .to_vec for SigningKey::from_bytes(), need to think about whats the best option here
    println!("Private Key:");
    println!("{:X?}", secret_key_bytes);
    println!("");

    // generate signing key from the secret key (same key, different type)
    let signing_key = SigningKey::from(&secret_key);
    // let signing_key = SigningKey::from_bytes(&secret_key_bytes).unwrap(); // another option of restoring the priv/signing key 
    let signing_key_bytes = signing_key.to_bytes();
    println!("Signing Key:");
    println!("{:X?}", signing_key_bytes);
    println!("");

    // test message to sign
    let message: Vec<u8> = vec![0x03, 0xED, 0x73, 0x45, 0xC0];
    println!("Message to Sign:");
    println!("{:X?}", message);
    println!("");

    // sign the message
    let signature: Signature = signing_key.sign(&message);
    println!("Signed Message:");
    println!("{:?}", signature);
    println!("");

    // generate public key from private key
    let public_key = secret_key.public_key();
    // convert to [u8] array in comprsesed format with 0x02 or 0x03 prefix (if y is even/odd) 
    let public_key_sec_bytes = public_key.to_sec1_bytes();
    println!("Compressed Public Key:");
    println!("{:X?}", public_key_sec_bytes);
    println!("");

    // generate verifying key from the public key (~same key (minus the odd/even bit and I think it needs the entire Y value), but the same content in general)
    let verifying_key = VerifyingKey::from(&public_key);
    let verifying_key_bytes = verifying_key.to_sec1_bytes();
    println!("Verifying Key:");
    println!("{:X?}", verifying_key_bytes);
    println!("");

    // assert that the signature matches the public key
    assert!(verifying_key.verify(&message, &signature).is_ok());
    println!("VerifyingKey verified Signed Message from SigningKey");
    println!("");

    // sha256 the compressed private key
    let mut sha256_hasher: Sha256 = Sha256::new();
    sha256_hasher.update(public_key_sec_bytes);
    let sha256_compressed_pub_key = sha256_hasher.finalize();
    println!("Sha256 Compressed Public Key:");
    println!("{:X?}", sha256_compressed_pub_key);
    println!("");

    // ripemd160  the sha(compressed pub key)
    let mut ripemd160_hasher = Ripemd160::new();
    ripemd160_hasher.update(sha256_compressed_pub_key);
    let ripemd_sha256_pub_key = ripemd160_hasher.finalize();
    println!("RIPEMD-160 Sha256 Compressed Public Key:");
    println!("{:X?}", ripemd_sha256_pub_key);
    println!("");

    // insert version bytes to front of ripe160(sha(pub key))
    let mut vec_of_ripe_sha_pub_key = ripemd_sha256_pub_key.to_vec();
    // version bytes to preface addresses with "BLoCK"
    // also adds a 1 at the end of BLoCK, which isn't necessarily great, but can be used as a kind of visual verseion number, if later addresses are generated
    // the last character is not guarnteed though, theoretically a large enough number could make this wrap to a 2 (or maybe not, infeasible and not really needed to test)
    // security of address is not sacrified due to additional size i.e. not subtracting from original address to add in prefix address = prefix (BLoCK1) + normal address size = normal address size + 6 (BLoCK1) 39bits total
    let version_bytes: Vec<u8> = vec![0x03, 0xED, 0x73, 0x45, 0xC0];

    version_bytes.iter().for_each(|item| {
        vec_of_ripe_sha_pub_key.push(*item);
    });
    
    // save the resulting version bytes + ripe(sha(pubkey))
    vec_of_ripe_sha_pub_key.rotate_right(version_bytes.len());
    println!("Version Bytes + RIPEMD-160 Sha256 Compressed Public Key:");
    println!("{:X?}", vec_of_ripe_sha_pub_key);
    println!("");

    // double sha256 the resulting version bytes + ripe(sha(pub key)) to get checksum 
    // create a copy of the version bytes + ripe(sha(pub))
    let copy_ripe_sha_pub = &vec_of_ripe_sha_pub_key[..];

    let mut sha256_hasher: Sha256 = Sha256::new();
    sha256_hasher.update(copy_ripe_sha_pub);
    let first_sha256 = sha256_hasher.finalize();
    println!("First Sha256 Version Bytes + RIPEMD-160 Sha256 Compressed Public Key:");
    println!("{:X?}", first_sha256);
    println!("");

    let mut sha256_hasher: Sha256 = Sha256::new();
    sha256_hasher.update(first_sha256);
    let second_sha256 = sha256_hasher.finalize();
    println!("Second Sha256 Version Bytes + RIPEMD-160 Sha256 Compressed Public Key:");
    println!("{:X?}", second_sha256);
    println!("");

    // push the first 4 bytes of the second sha as the checksum at the end of the version bytes + ripe(sha(pub))
    vec_of_ripe_sha_pub_key.push(second_sha256[0]);
    vec_of_ripe_sha_pub_key.push(second_sha256[1]);
    vec_of_ripe_sha_pub_key.push(second_sha256[2]);
    vec_of_ripe_sha_pub_key.push(second_sha256[3]);
    println!("Version Bytes + RIPEMD-160 Sha256 Compressed Public Key + Checksum:");
    println!("{:X?}", vec_of_ripe_sha_pub_key);
    println!("");

    // convert to base58 encoded string to get block Addres
    let encoded = bs58::encode(vec_of_ripe_sha_pub_key).into_string();
    println!("block Address");
    println!("{:?}", encoded);
    println!("");
}
