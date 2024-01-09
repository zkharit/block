use k256::SecretKey;
use rand_core::OsRng;
use sha2::{Sha256, Digest};
use ripemd::Ripemd160;

fn main() {
    // Test secret key found on: https://en.bitcoin.it/wiki/Technical_background_of_version_1_Bitcoin_addresses#How_to_create_Bitcoin_Address
    // Need more test vectors with some edge cases, leading/trailing 0's etc
    // Import key, as well as sued for static key testing
    // let secret_key_material: &[u8] = &[24, 225, 74, 123, 106, 48, 127, 66, 106, 148, 248, 17, 71, 1, 231, 200, 231, 116, 231, 249, 164, 126, 44, 32, 53, 219, 41, 162, 6, 50, 23, 37];
    // let secret_key = SecretKey::from_bytes(secret_key_material.into()).unwrap();

    // generate private key
    let secret_key = SecretKey::random(&mut OsRng);
    let secret_key_bytes = secret_key.to_bytes();
    println!("Private Key:");
    println!("{:X?}", secret_key_bytes);
    println!("");

    // println!("{}", secret_key.to_nonzero_scalar());

    // generate public key from private key
    let public_key = secret_key.public_key();
    //let public_key_encoded_compressed = public_key.to_encoded_point(true);

    // convert to [u8] array in comprsesed format with 0x02 or 0x03 prefix (if y is even/odd) 
    let public_key_sec_bytes = public_key.to_sec1_bytes();
    println!("Compressed Public Key:");
    //println!("{:?}", public_key_encoded_compressed.to_string());
    println!("{:?}", public_key_sec_bytes);
    println!("");

    // sha256 the compressed private key
    let mut sha256_hasher: Sha256 = Sha256::new();

    sha256_hasher.update(public_key_sec_bytes);
    let sha256_compressed_pub_key = sha256_hasher.finalize();
    println!("Sha256 Compressed Public Key:");
    println!("{:?}", sha256_compressed_pub_key);
    println!("");

    // ripemd160  the sha(compressed pub key)
    let mut ripemd160_hasher = Ripemd160::new();
    ripemd160_hasher.update(sha256_compressed_pub_key);
    let ripemd_sha256_pub_key = ripemd160_hasher.finalize();
    println!("RIPEMD-160 Sha256 Compressed Public Key:");
    println!("{:?}", ripemd_sha256_pub_key);
    println!("");

    // insert version bytes to front of ripe160(sha(pub key))
}

