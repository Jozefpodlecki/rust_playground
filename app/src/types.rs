use std::path::PathBuf;

pub struct RunArgs {
    pub cipher_key: Vec<u8>,
    pub aes_xor_key: Vec<u8>,
    pub lpk_dir: String,
    pub output_path: PathBuf,
    pub exe_path: String,
    pub exe_args: Vec<String>,
}