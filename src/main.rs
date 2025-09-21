
use std::path::Path;

use clap::Parser;
use lib::{image_utils::save_images, pdf::create_pdfium};

#[derive(Parser)]
#[command(name = "pdf_diff")]
#[command(about = "A tool for comparing PDF documents by generating visual diffs")]
#[command(version = "0.1.0")]
struct Args {
    /// Path to the old PDF file
    #[arg(short = 'o', long = "old", help = "Path to the old PDF file",)]
    old_pdf: String,

    /// Path to the new PDF file
    #[arg(short = 'n', long = "new", help = "Path to the new PDF file")]
    new_pdf: String,

    /// Output directory for diff images
    #[arg(short = 'd', long = "output-dir", default_value = "output", help = "Directory to save diff images")]
    output_dir: String,

    /// DPI for rendering (higher = better quality, slower processing)
    #[arg(long = "dpi", default_value = "300", help = "DPI for PDF rendering")]
    dpi: f32,

    /// Diff sensitivity (0.0-1.0, lower = more sensitive)
    #[arg(long = "sensitivity", default_value = "0.12", help = "Diff sensitivity threshold")]
    sensitivity: f32,

    /// Verbose output
    #[arg(short = 'v', long = "verbose", help = "Enable verbose output")]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    if args.verbose {
        println!("PDF Diff Tool v0.1.0");
        println!("Old PDF: {}", args.old_pdf);
        println!("New PDF: {}", args.new_pdf);
        println!("Output directory: {}", args.output_dir);
        println!("DPI: {}", args.dpi);
        println!("Sensitivity: {}", args.sensitivity);
        println!();
    }

    let path_old = Path::new(&args.old_pdf);
    let path_new = Path::new(&args.new_pdf);

    // Validate input files exist
    if !path_old.exists() {
        eprintln!("Error: Old PDF file does not exist: {}", args.old_pdf);
        std::process::exit(1);
    }

    if !path_new.exists() {
        eprintln!("Error: New PDF file does not exist: {}", args.new_pdf);
        std::process::exit(1);
    }

    if args.verbose {
        println!("Creating PDFium instance...");
    }

    let pdfium = match create_pdfium() {
        Ok(pdfium) => pdfium,
        Err(e) => {
            eprintln!("Error creating PDFium instance: {}", e);
            std::process::exit(1);
        }
    };

    if args.verbose {
        println!("Loading PDF documents...");
    }

    let (old_document, new_document) = match lib::pdf::load_pdf_documents(&pdfium, path_old, path_new) {
        Ok((old, new)) => {
            if args.verbose {
                println!("Loaded {} pages from old PDF", old.pages().len());
                println!("Loaded {} pages from new PDF", new.pages().len());
            }
            (old, new)
        },
        Err(e) => {
            eprintln!("Error loading PDF files: {}", e);
            std::process::exit(1);
        }
    };

    if args.verbose {
        println!("Converting PDF pages to images...");
    }

    let images = match lib::pdf::create_images_from_pdf(&old_document, &new_document, args.dpi) {
        Ok(images) => {
            if args.verbose {
                println!("Generated {} image pairs", images.len());
            }
            images
        },
        Err(e) => {
            eprintln!("Error creating images from PDF: {}", e);
            std::process::exit(1);
        }
    };

    if args.verbose {
        println!("Generating diff images...");
    }

    let diff_images = match lib::image_utils::diff_images(images, args.sensitivity) {
        Ok(images) => {
            if args.verbose {
                println!("Generated {} diff images", images.len());
            }
            images
        },
        Err(e) => {
            eprintln!("Error diffing images: {}", e);
            std::process::exit(1);
        }
    };

    if args.verbose {
        println!("Saving images to '{}'...", args.output_dir);
    }

    match save_images(diff_images, &args.output_dir) {
        Ok(()) => {
            if args.verbose {
                println!("Successfully saved all diff images!");
            } else {
                println!("Diff images saved to '{}'", args.output_dir);
            }
        },
        Err(e) => {
            eprintln!("Error saving images: {}", e);
            std::process::exit(1);
        }
    }
}




