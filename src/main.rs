use image;
use image::{imageops::FilterType, DynamicImage, GrayImage};
use bardecoder;
use sha2::{Digest, Sha256};

fn resize_image(image: &DynamicImage, new_width: u32, new_height: u32) -> DynamicImage {
    image.resize(new_width, new_height, FilterType::Nearest)
}

fn decode_qr_image(image: &DynamicImage) -> Option<String> {
    let decoder = bardecoder::default_decoder();
    let results = decoder.decode(image);

    if results.is_empty() {
        println!("No QR codes found in the image.");
        return None;
    }

    for result in &results {
        match result {
            Ok(content) => {
                println!("QR successfully decoded from image!");
                return Some(content.clone());
            }
            Err(e) => println!("Failed to decode QR code: {}", e),
        }
    }

    None
}

fn extract_bits_from_separator(
    img_gray: &GrayImage,
    x_start: u32,
    y_start: u32,
    x_stop: u32,
    y_stop: u32,
    direction: &str,
) -> Vec<u8> {
    let mut bits = Vec::new();
    if direction == "vertical" {
        if y_start <= y_stop {
            for y in y_start..=y_stop {
                let pixel = img_gray.get_pixel(x_start, y);
                bits.push(if pixel[0] == 255 { 0 } else { 1 });
            }
        } else {
            for y in (y_stop..=y_start).rev() {
                let pixel = img_gray.get_pixel(x_start, y);
                bits.push(if pixel[0] == 255 { 0 } else { 1 });
            }
        }
    } else if direction == "horizontal" {
        if x_start <= x_stop {
            for x in x_start..=x_stop {
                let pixel = img_gray.get_pixel(x, y_start);
                bits.push(if pixel[0] == 255 { 0 } else { 1 });
            }
        } else {
            for x in (x_stop..=x_start).rev() {
                let pixel = img_gray.get_pixel(x, y_start);
                bits.push(if pixel[0] == 255 { 0 } else { 1 });
            }
        }
    }
    bits
}

fn verify_qr_overlay(original_image: &DynamicImage, decoded_message: &str) -> bool {
    let img_gray = original_image.to_luma8();
    let (mut width, height) = (img_gray.width(), img_gray.height());

    if width != height || width < 21 {
        println!("Image resolution is incorrect");
        return false;
    }

    // Hash the decoded message
    let mut hasher = Sha256::new();
    hasher.update(decoded_message.as_bytes());
    let hash = hasher.finalize();
    let bit_string: Vec<u8> = hash
        .iter()
        .flat_map(|byte| (0..8).rev().map(move |bit| (byte >> bit) & 1))
        .collect();
    let reference_bits = &bit_string[0..45]; // First 45 bits

    let finder_size = 7; // Finder pattern is always 7x7 modules
    width -= 1;

    // Extract overlay bits from each separator strip
    let mut overlay_bits = Vec::new();
    overlay_bits.extend(extract_bits_from_separator(
        &img_gray,
        finder_size as u32,
        width as u32,
        finder_size as u32,
        (width - finder_size) as u32,
        "vertical",
    ));
    overlay_bits.extend(extract_bits_from_separator(
        &img_gray,
        (finder_size - 1) as u32,
        (width - finder_size) as u32,
        0,
        (width - finder_size) as u32,
        "horizontal",
    ));
    overlay_bits.extend(extract_bits_from_separator(
        &img_gray,
        0,
        finder_size as u32,
        finder_size as u32,
        finder_size as u32,
        "horizontal",
    ));
    overlay_bits.extend(extract_bits_from_separator(
        &img_gray,
        finder_size as u32,
        (finder_size - 1) as u32,
        finder_size as u32,
        0,
        "vertical",
    ));
    overlay_bits.extend(extract_bits_from_separator(
        &img_gray,
        (width - finder_size) as u32,
        0,
        (width - finder_size) as u32,
        finder_size as u32,
        "vertical",
    ));
    overlay_bits.extend(extract_bits_from_separator(
        &img_gray,
        (width - finder_size + 1) as u32,
        finder_size as u32,
        width as u32,
        finder_size as u32,
        "horizontal",
    ));

    // Compare the extracted bits with the reference bits
    overlay_bits == reference_bits
}

fn process_qr_code(path_to_image: &str) {
    // Load the original QR code image
    let original_image = image::open(path_to_image)
        .expect("Failed to open the QR code image");

    // Resize the QR code image for decoding
    let resized_image = resize_image(&original_image, 1000, 1000);

    // Decode the QR code
    let decoded_message = match decode_qr_image(&resized_image) {
        Some(message) => message,
        None => {
            println!("Failed to decode the QR code.");
            return;
        }
    };
    println!("Decoded message: {}", decoded_message);

    // Verify the QR code overlay
    let is_valid = verify_qr_overlay(&original_image, &decoded_message);
    println!(
        "Verification: {}",
        if is_valid { "valid ✅" } else { "invalid ❌" }
    );
}

fn main() {
    // Update the path to the QR with overlay
    let image_path = "fake_qr_with_separator_overlay.png";

    process_qr_code(image_path);
}
