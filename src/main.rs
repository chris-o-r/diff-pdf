
use std::path::{self};

use lib::image_utils::save_images;
use pdfium::set_library_location;

const LIBRARY_LOCATION: &str = "./pdfium-mac-arm64/lib/";

fn main() {
    set_library_location(LIBRARY_LOCATION);

    let path_old = path::Path::new("./samples/sample_3.pdf");
    let path_new = path::Path::new("./samples/sample_4.pdf");

    let (old_document, new_document) = match lib::pdf::load(path_old, path_new) {
        Ok((old, new)) => (old, new),
        Err(e) => {
            eprintln!("Error loading PDF files: {}", e);
            return;
        }
    };

    let images = match lib::pdf::create_images_from_pdf(&old_document, &new_document) {
        Ok(images) => images,
        Err(e) => {
            eprintln!("Error creating images from PDF: {}", e);
            return;
        }
    };

    let diff_images = match lib::image_utils::diff_images(images) {
        Ok(images) => images,
        Err(e) => {
            eprintln!("Error diffing images: {}", e);
            return;
        }
    };


    // Save diff images to files
    save_images(diff_images, "output").unwrap();
}




