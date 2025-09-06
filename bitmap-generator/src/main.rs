use image::{DynamicImage, GenericImageView, imageops::FilterType, RgbImage, Rgb};
use std::path::Path;

fn convert_and_resize(input: &str, width: u32, height: u32, output: &str) {
    // Load the WebP image
    let exe_path = std::env::current_exe().unwrap();
    let base_path = exe_path.parent().unwrap();
    let path = base_path.join(input);
    println!("{:?}", path);
    let img = image::open(path).expect("Failed to open input image");

    // Resize to target dimensions
    let resized = img.resize_exact(width, height, FilterType::Lanczos3);

    // Convert to RGB and save as BMP
    let rgb_image: RgbImage = resized.to_rgb8();
    rgb_image.save(output).expect("Failed to save BMP image");
}

fn main() {
    convert_and_resize("banner.jpg", 493, 58, "banner.bmp");

    convert_and_resize("dialog.jpg", 493, 312, "dialog.bmp");

    println!("Both images have been converted and saved as BMP!");
}