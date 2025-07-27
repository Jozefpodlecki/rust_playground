   // println!("{cipher_key:?}");
    // let mut blowfish = Blowfishv2::new(&cipher_key);
    // // let mut blowfish = Blowfish::new(&cipher_key);
    // let decrypted = blowfish.decrypt_ecb(&entry_bytes);

    // let formatted = decrypted[..20].iter()
    //     .map(|b| format!("{:02X}", b))
    //     .collect::<Vec<_>>()
    //     .join("-");
    // println!("{}", formatted);

    // println!("{}", decrypted.len());

        //  var offset = entriesSize + 8;
//     MaxLength = reader.ReadInt32();
// Length = reader.ReadInt32();
// ContentType = GetContentType(FileName);
// StorageType = reader.ReadInt32() == 0 ? LpkEntryStorageType.Encrypted : LpkEntryStorageType.Compressed;
// Offset = offset;

    // println!("Relative path length: {}, File name: {}", relative_file_path_length, file_name);
    // let cipher = Ecb::<Blowfish, NoPadding>::new_from_slices(&cipher_key, &[]).unwrap();

    // let blowfish = Blowfish::new_varkey(b"your_blowfish_key").unwrap();
    // let decrypted_bytes = blowfish.decrypt_ecb(&entry_bytes);

    // // 3) Create a stream + reader for decrypted bytes
    // let mut stream = Cursor::new(decrypted_bytes);

    // // 4) Setup AES cryptographic object (key/iv can be placeholder for now)
    // let key = [0u8; 32]; // 256 bits
    // let iv = [0u8; 16];  // 128 bits
    // let aes = Arc::new(Aes256::new_from_slice(&key).unwrap());
    // let crypto_object = Arc::new(Cbc::<Aes256, NoPadding>::new_from_slices(&key, &iv).unwrap());

    // // 5) Track offset
    // let offset = entries_size as u64 + 8;

    // let mut entry = LpkEntry::new(
    //         Arc::clone(&aes),
    //         Arc::new(blowfish.clone()),
    //         File::open("your_input_file.lpk")?,
    //         &mut stream,
    //         offset,
    //         i,
    //     )?;

    // let entry = LpkEntry::new(
    //     cryptographic_object, blowfish, lpk_reader, reader, offset, file_order)
