use std::path::{Path};

use std::fmt;
use std::error::Error;
#[allow(unused_imports)]
use image::{DynamicImage, GenericImageView};
use pdfium_render::prelude::{PdfDocument, PdfRenderConfig, Pdfium};

 
#[derive(Debug)]
pub struct PdfError {
    message: String
}

impl fmt::Display for PdfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for PdfError {}

pub fn create_pdfium() -> Result<Pdfium, PdfError> {
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./pdfium-mac-arm64/lib/"))
            .map_err(|e| PdfError {
                message: format!("Failed to bind to PDFium library: {:?}", e),
            })?
    );
    Ok(pdfium)
}

pub fn load_pdf_documents<'a>(
    pdfium: &'a Pdfium,
    old_pdf_path: &Path, 
    new_pdf_path: &Path
) -> Result<(PdfDocument<'a>, PdfDocument<'a>), PdfError> {
    if !old_pdf_path.exists() || !old_pdf_path.is_file() {
        return Err(PdfError {
            message: format!("Old PDF file does not exist: {:?}", old_pdf_path),
        });
    }

    if !new_pdf_path.exists() || !new_pdf_path.is_file() {
        return Err(PdfError {
            message: format!("New PDF file does not exist: {:?}", new_pdf_path),
        });
    }

    let old_document = pdfium.load_pdf_from_file(old_pdf_path, None).map_err(|e| PdfError {
        message: format!("Failed to load old PDF file: {:?}", e),
    })?;
    let new_document = pdfium.load_pdf_from_file(new_pdf_path, None).map_err(|e| PdfError {
        message: format!("Failed to load new PDF file: {:?}", e),
    })?;

    Ok((old_document, new_document))
}

pub fn create_images_from_pdf(
    old_document: &PdfDocument,
    new_document: &PdfDocument,
    dpi: f32,
) -> Result<Vec<(Option<DynamicImage>, Option<DynamicImage>)>, PdfError> {
    let mut result = Vec::<(Option<DynamicImage>, Option<DynamicImage>)>::new();
    
    for index in 0..new_document.pages().len() {
        let new_page = new_document.pages().get(index).map_err(|e| PdfError {
            message: format!("Failed to get page {} from new PDF: {:?}", index, e),
        })?;
        let new_img = get_image_from_page(&new_page, dpi)?;

        let old_page = old_document.pages().get(index).ok();

        let old_image = match old_page {
            Some(page) => Some(get_image_from_page(&page, dpi)?),
            None => None,
        };

        result.push((old_image, Some(new_img)));
    }

    Ok(result)
}


fn get_image_from_page(page: &pdfium_render::prelude::PdfPage, dpi: f32) -> Result<DynamicImage, PdfError> {
      let render_config = PdfRenderConfig::new()
            .set_target_width((page.width().value * dpi / 72.0).round() as i32)
            .set_maximum_height((page.height().value * dpi / 72.0).round() as i32);

    Ok(page.render_with_config(&render_config).map_err(|e| PdfError {
        message: format!("Failed to render page to image: {:?}", e),
    })?.as_image())

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_create_pdfium() {
        let result = create_pdfium();
        assert!(result.is_ok(), "Failed to create Pdfium instance: {:?}", result.err());
    }

    #[test]
    fn test_load_pdf_documents() {
        let pdfium = create_pdfium().expect("Failed to create Pdfium instance");
        let old_path = Path::new("./samples/old.pdf");
        let new_path = Path::new("./samples/new.pdf");

        let result = load_pdf_documents(&pdfium, old_path, new_path);
        assert!(result.is_ok(), "Failed to load PDF documents: {:?}", result.err());

        let (old_doc, new_doc) = result.unwrap();
        assert!(old_doc.pages().len() > 0, "Old document should have pages");
        assert!(new_doc.pages().len() > 0, "New document should have pages");
    }

    #[test]
    fn test_load_pdf_documents_nonexistent_file() {
        let pdfium = create_pdfium().expect("Failed to create Pdfium instance");
        let old_path = Path::new("./nonexistent.pdf");
        let new_path = Path::new("./samples/new.pdf");

        let result = load_pdf_documents(&pdfium, old_path, new_path);
        assert!(result.is_err(), "Should fail for nonexistent file");
        
        let error = result.err().unwrap();
        assert!(error.to_string().contains("does not exist"));
    }

    #[test]
    fn test_create_images_from_pdf() {
        let pdfium = create_pdfium().expect("Failed to create Pdfium instance");
        let old_path = Path::new("./samples/old.pdf");
        let new_path = Path::new("./samples/new.pdf");

        let (old_doc, new_doc) = load_pdf_documents(&pdfium, old_path, new_path)
            .expect("Failed to load PDF documents");

        let result = create_images_from_pdf(&old_doc, &new_doc, 300.0);
        assert!(result.is_ok(), "Failed to create images from PDF: {:?}", result.err());

        let images = result.unwrap();
        assert!(images.len() > 0, "Should generate at least one image pair");

        // Check that we have valid image data
        for (i, (_old_img, new_img)) in images.iter().enumerate() {
            assert!(new_img.is_some(), "New image should exist for page {}", i);
            
            if let Some(img) = new_img {
                let (width, height) = img.dimensions();
                assert!(width > 0 && height > 0, "Image dimensions should be positive for page {}", i);
            }
        }
    }

    #[test]
    fn test_get_image_from_page() {
        let pdfium = create_pdfium().expect("Failed to create Pdfium instance");
        let pdf_path = Path::new("./samples/old.pdf");

        let (doc, _) = load_pdf_documents(&pdfium, pdf_path, pdf_path)
            .expect("Failed to load PDF document");

        let page = doc.pages().get(0).expect("Failed to get first page");
        let result = get_image_from_page(&page, 300.0);
        
        assert!(result.is_ok(), "Failed to render page to image: {:?}", result.err());
        
        let image = result.unwrap();
        let (width, height) = image.dimensions();
        assert!(width > 0 && height > 0, "Image should have positive dimensions");
        // Remove the width/height constraints since they depend on DPI and page size
    }

    #[test]
    fn test_different_page_counts() {
        let pdfium = create_pdfium().expect("Failed to create Pdfium instance");
        let old_path = Path::new("./samples/old.pdf");
        let new_path = Path::new("./samples/new.pdf");

        let (old_doc, new_doc) = load_pdf_documents(&pdfium, old_path, new_path)
            .expect("Failed to load PDF documents");

        let result = create_images_from_pdf(&old_doc, &new_doc, 300.0);
        assert!(result.is_ok(), "Should handle different page counts");

        let images = result.unwrap();
        // Should process as many pages as the new document has
        assert_eq!(images.len(), new_doc.pages().len() as usize, "Should process all pages from new document");
    }
}
