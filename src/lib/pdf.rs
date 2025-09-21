use std::path::{Path};

use std::fmt;
use std::error::Error;
use image::DynamicImage;
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
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./pdfium-mac-arm64/lib/")).unwrap()
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
) -> Result<Vec<(Option<DynamicImage>, Option<DynamicImage>)>, PdfError> {
    let mut result = Vec::<(Option<DynamicImage>, Option<DynamicImage>)>::new();
    
    for index in 0..new_document.pages().len() {
        let new_page = new_document.pages().get(index).map_err(|e| PdfError {
            message: format!("Failed to get page {} from new PDF: {:?}", index, e),
        })?;
        let new_img = get_image_from_page(&new_page)?;

        let old_page = old_document.pages().get(index).ok();

        let old_image = match old_page {
            Some(page) => Some(get_image_from_page(&page)?),
            None => None,
        };

        result.push((old_image, Some(new_img)));
    }

    Ok(result)
}


fn get_image_from_page(page: &pdfium_render::prelude::PdfPage) -> Result<DynamicImage, PdfError> {
      let render_config = PdfRenderConfig::new()
            .set_target_width(2000)
            .set_maximum_height(2000);

    Ok(page.render_with_config(&render_config).map_err(|e| PdfError {
        message: format!("Failed to render page to image: {:?}", e),
    })?.as_image())

}
