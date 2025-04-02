use std::{env, fs, path::PathBuf};
use std::io::Write;
use dotenvy::dotenv;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use webp::{Encoder, WebPMemory};

fn main() {
    dotenv().unwrap();
   
    convert_parallel();
}

fn convert_parallel() {
    let source_dir = env::var("SOURCE_PATH").expect("SOURCE_PATH is not set");
    let destination_dir = env::var("DESTINATION_PATH").expect("DESTINATION_PATH is not set");

    fs::create_dir_all(&destination_dir).expect("Failed to create destination directory");

    let entries: Vec<PathBuf> = fs::read_dir(&source_dir)
        .expect("Failed to read source directory")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("png"))
        .collect();

    entries.par_iter().for_each(|path| convert_to_webp(path, &destination_dir));
}

fn convert_sync() {
    let source_dir = env::var("SOURCE_PATH").expect("SOURCE_PATH is not set");
    let destination_dir = env::var("DESTINATION_PATH").expect("DESTINATION_PATH is not set");

    fs::create_dir_all(&destination_dir).expect("Failed to create destination directory");

    let entries = fs::read_dir(&source_dir).expect("Failed to read source directory");

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("png") {
                convert_to_webp(&path, &destination_dir);
            }
        }
    }
}

fn convert_to_webp(input_path: &PathBuf, output_dir: &str) {
    let image = image::open(input_path).expect("Failed to open image");

    let encoder: Encoder = Encoder::from_image(&image).expect("Failed to encode image");
    let webp: WebPMemory = encoder.encode(90f32);

    let mut output_path = PathBuf::from(output_dir);
    output_path.push(input_path.file_stem().unwrap());
    output_path.set_extension("webp");

    let mut output_file = std::fs::File::create(&output_path).expect("Failed to create output file");
    output_file.write_all(&webp).expect("Failed to write output file");

    println!("Converted: {} -> {}", input_path.display(), output_path.display());
}