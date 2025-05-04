
use chacha20::ChaCha20;
use chacha20::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
use hex::encode;

fn enncrypt_and_decrypt() {
    let key = [0x42; 32];
    let nonce = b"unique nonce";

    let plaintext = b"encrypted";
    let mut buffer = plaintext.to_vec();
    let nonce = [0x24; 12];
    
    let mut cipher = ChaCha20::new(&key.into(), &nonce.into());
    cipher.apply_keystream(&mut buffer);

    println!("Encrypted: {}", encode(&buffer));

    // Decrypting
    // let mut cipher = ChaCha20::new(key.into(), nonce.into());
    cipher.seek(0u32);
    cipher.apply_keystream(&mut buffer);
    println!("Decrypted: {}", String::from_utf8_lossy(&buffer));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cha20_encrypt() {
        enncrypt_and_decrypt();
    }
}
