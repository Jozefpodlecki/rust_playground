use image::{imageops, DynamicImage, GenericImageView, ImageReader};
use std::fs::File;
use std::io::Write;
use webp::{Encoder, WebPMemory};

fn main() {
    // let image = ImageReader::open("image.png").unwrap();
    let executable_path = std::env::current_exe().unwrap();
    let executable_directory = executable_path.parent().unwrap().to_path_buf();
    let mut path = executable_directory.clone();
    path = path.join("image.png");
    println!("{:?}", path);

    let image = image::open(path).unwrap();

    let (w, h) = image.dimensions();
    let size_factor = 1.0;
    let image: DynamicImage = image::DynamicImage::ImageRgba8(imageops::resize(
        &image,
        (w as f64 * size_factor) as u32,
        (h as f64 * size_factor) as u32,
        imageops::FilterType::Triangle,
    ));
    let encoder: Encoder = Encoder::from_image(&image).unwrap();
    let webp: WebPMemory = encoder.encode(90f32);
    
    let mut output_path = executable_directory.clone();
    output_path = output_path.join("image.webp");

    let mut output = File::create(output_path).unwrap();
    output.write_all(&webp).unwrap();
}
