use std::path::{Path};

use std::fmt;
use std::error::Error;
use image::DynamicImage;
use pdfium::{PdfiumDocument, PdfiumPage, PdfiumRenderConfig};

use crate::image_utils;

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

pub fn load(old_pdf_path: &Path, new_pdf_path: &Path) -> Result<(PdfiumDocument, PdfiumDocument), PdfError> {
   
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

        let old_document = PdfiumDocument::new_from_path(old_pdf_path, None).map_err(|e| PdfError {
        message: format!("Failed to load old PDF file: {:?}", e),
    })?;
    let new_document = PdfiumDocument::new_from_path(new_pdf_path, None).map_err(|e| PdfError {
        message: format!("Failed to load new PDF file: {:?}", e),
    })?;

    Ok((old_document, new_document))
}

pub fn create_images_from_pdf(
    old_document: &PdfiumDocument,
    new_document: &PdfiumDocument,
) -> Result<Vec<(Option<DynamicImage>, Option<DynamicImage>)>, PdfError> {
    let mut result = Vec::<(Option<DynamicImage>, Option<DynamicImage>)>::new();
    // Implementation for creating images from the PDF document

    for (index, new_page) in new_document.pages().enumerate() {
        let new_img = get_image_from_page(&new_page.map_err(|e| PdfError {
            message: format!("Failed to get page from new PDF: {:?}", e),
        })?)?;

        let old_page = old_document.pages().get(index.try_into().unwrap()).ok();

        let old_image = match old_page {
            Some(page) => Some(get_image_from_page(&page)?),
            None => None,
        };

        result.push((old_image, Some(new_img)));
    }

    Ok(result)
}


fn get_image_from_page(page: &PdfiumPage) -> Result<DynamicImage, PdfError> {
    const DPI: f32 = 300.0; // Define a constant for DPI to ensure consistency

    // Use the page's actual size in points (1/72 inch) and render at a higher DPI for better resolution.
    let width = (page.width() * DPI / 72.0).round() as u16;
    let height = (page.height() * DPI / 72.0).round() as u16;

    let render_config = PdfiumRenderConfig::new()
        .with_width(width.into())
        .with_height(height.into())
        .with_format(pdfium::PdfiumBitmapFormat::Bgra)
        .with_scale(1.0); // Keep scale at 1.0, use width/height for resolution

    let img = page.render(&render_config).map_err(|e| PdfError {
        message: format!("Failed to render page to image: {:?}", e),
    })?
    .as_rgba8_image()
    .map_err(|e| PdfError {
        message: format!("Failed to convert rendered page to image: {:?}", e),
    })?;
    Ok(image_utils::crop_to_content(&img))
}
