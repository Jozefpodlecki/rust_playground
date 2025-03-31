use std::fs::File;
use std::io::Write;
use webp::{Encoder, WebPMemory};

fn main() {
    let executable_path = std::env::current_exe().unwrap();
    let executable_directory = executable_path.parent().unwrap().to_path_buf();
    let mut path = executable_directory.clone();
    path = path.join("image.png");

    let image = image::open(path).unwrap();
    let encoder: Encoder = Encoder::from_image(&image).unwrap();
    let webp: WebPMemory = encoder.encode(90f32);
    
    let mut output_path = executable_directory.clone();
    output_path = output_path.join("image.webp");

    let mut output = File::create(output_path).unwrap();
    output.write_all(&webp).unwrap();
}
