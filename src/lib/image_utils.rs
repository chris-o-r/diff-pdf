use diff_img::lcs_diff;
use image::{DynamicImage, GenericImageView};
// Crop a DynamicImage to its non-white content (tolerant to near-white)
pub fn crop_to_content(img: &DynamicImage) -> DynamicImage {
    let (width, height) = img.dimensions();
    let rgba = img.to_rgba8();

    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut found = false;

    // Tolerance for "white" (255,255,255,255). Allow a little off-white.
    let tolerance = 10u8;

    for y in 0..height {
        for x in 0..width {
            let pixel = rgba.get_pixel(x, y);
            // If not close to white, mark as content
            if !(pixel[0] >= 255 - tolerance && pixel[1] >= 255 - tolerance && pixel[2] >= 255 - tolerance && pixel[3] >= 255 - tolerance) {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                found = true;
            }
        }
    }

    if found {
        img.crop_imm(min_x, min_y, max_x - min_x + 1, max_y - min_y + 1)
    } else {
        img.clone()
    }
}

pub fn save_images(
    images: Vec<DynamicImage>,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use image::ImageFormat;
    use std::fs::File;
    use std::io::BufWriter;

    std::fs::create_dir_all(output_dir)?;

    for (i, img) in images.iter().enumerate() {
        let output_path = format!("{}/diff_page_{}.png", output_dir, i + 1);
        let file = File::create(&output_path)?;
        let w = BufWriter::new(file);
    img.write_to(&mut BufWriter::new(w), ImageFormat::Png)?;
        println!("Saved diff image to {}", output_path);
    }

    Ok(())
}
 
pub fn diff_images(
    images: Vec<(Option<DynamicImage>, Option<DynamicImage>)>,
) -> Result<Vec<DynamicImage>, Box<dyn std::error::Error>> {
    let mut diff = vec![];

    for (old_image, new_image) in &images {
        match (old_image, new_image) {
            (Some(old), Some(new)) => {
                let mut old = old.clone();
                let mut new = new.clone();

                let diff_ratio = diff_img::calculate_diff_ratio(&old, &new);
                if diff_ratio > 0.0 {
                    let diff_image = lcs_diff(&mut old, &mut new, 0.12)?;
                    diff.push(diff_image);
                }

                diff.push(new);
            }
            (None, Some(new)) => {
                diff.push(new.clone());
            }
            (Some(old), None) => {
                diff.push(old.clone());
            }
            (None, None) => {}
        }
    }

    Ok(diff)
}
