use super::CertificateData;
use crate::certificates::error::{CertificateError, Result};
use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use base64::Engine as _;
use base64::engine::general_purpose;
use image::{DynamicImage, ImageBuffer, ImageFormat, ImageReader, Luma, Rgba, RgbaImage, imageops};
use imageproc::drawing::draw_text_mut;
use imageproc::rect::Rect;
use lazy_static::lazy_static;
use plot_icon::generate_png;
use qrcode::{EcLevel, QrCode, types::QrError as QrCodeError};
use std::io::Cursor;

// Define color constants
const BORDER_COLOR: Rgba<u8> = Rgba([0, 82, 165, 255]); // Deep Blue
const BACKGROUND_COLOR: Rgba<u8> = Rgba([255, 253, 245, 255]); // Soft Cream
const TEXT_COLOR: Rgba<u8> = Rgba([51, 51, 51, 255]); // Dark Gray
const HIGHLIGHT_COLOR: Rgba<u8> = Rgba([0, 121, 193, 255]); // Bright Blue

// Define font constants
const TITLE_FONT_SIZE: f32 = 48.0;
const BODY_FONT_SIZE: f32 = 24.0;
const SMALL_FONT_SIZE: f32 = 18.0;

lazy_static! {
    static ref TITLE_FONT: FontRef<'static> =
        FontRef::try_from_slice(include_bytes!("../../../assets/Montserrat-Bold.ttf"))
            .expect("Failed to load title font");
    static ref BODY_FONT: FontRef<'static> =
        FontRef::try_from_slice(include_bytes!("../../../assets/Lora-Regular.ttf"))
            .expect("Failed to load body font");
}

pub fn create_certificate(
    certificate_data: &CertificateData,
    url: &str,
    issuer: &str,
) -> Result<DynamicImage> {
    let cert_width = 1200;
    let cert_height = 900;
    let border_thickness = 30u32;
    let qr_code_size = 250;

    // Create QR code
    let qr_code =
        QrCode::with_error_correction_level(url, EcLevel::H).map_err(|e: QrCodeError| {
            CertificateError::ImageProcessingError(format!("Failed to create QR code: {}", e))
        })?;

    let qr_code_image = qr_code.render::<Luma<u8>>().quiet_zone(false).build();
    let qr_code_image_rgba =
        ImageBuffer::from_fn(qr_code_image.width(), qr_code_image.height(), |x, y| {
            let pixel = qr_code_image.get_pixel(x, y);
            if pixel[0] == 0 {
                Rgba([0, 0, 0, 255])
            } else {
                Rgba([255, 255, 255, 255])
            }
        });
    let resized_qr_code_image = imageops::resize(
        &qr_code_image_rgba,
        qr_code_size,
        qr_code_size,
        imageops::FilterType::Nearest,
    );

    // Create base image
    let mut cert_image = RgbaImage::new(cert_width, cert_height);

    // Draw border and background
    imageproc::drawing::draw_filled_rect_mut(
        &mut cert_image,
        Rect::at(0, 0).of_size(cert_width, cert_height),
        BORDER_COLOR,
    );
    imageproc::drawing::draw_filled_rect_mut(
        &mut cert_image,
        Rect::at(border_thickness as i32, border_thickness as i32).of_size(
            cert_width - 2 * border_thickness,
            cert_height - 2 * border_thickness,
        ),
        BACKGROUND_COLOR,
    );

    // Load and draw logo
    let logo_bytes = include_bytes!("../../../assets/favicon.png");
    let logo_image = ImageReader::new(Cursor::new(logo_bytes))
        .with_guessed_format()
        .map_err(|e| {
            CertificateError::ImageProcessingError(format!(
                "Failed to guess logo image format: {}",
                e
            ))
        })?
        .decode()
        .map_err(|e| {
            CertificateError::ImageProcessingError(format!("Failed to decode logo image: {}", e))
        })?;

    let scaled_logo_image = imageops::resize(&logo_image, 120, 120, imageops::FilterType::Lanczos3);
    image::imageops::overlay(&mut cert_image, &scaled_logo_image, 50, 50);

    // Generate and draw identicon
    let identicon_image = {
        let data: Vec<u8> =
            generate_png(certificate_data.to_base64()?.as_bytes(), 90).map_err(|e| {
                CertificateError::ImageProcessingError(format!(
                    "Failed to generate identicon: {}",
                    e
                ))
            })?;

        let mut image = ImageReader::new(Cursor::new(data));
        image.set_format(image::ImageFormat::Png);

        image.decode().map_err(|e| {
            CertificateError::ImageProcessingError(format!("Failed to decode identicon: {}", e))
        })?
    };

    image::imageops::overlay(
        &mut cert_image,
        &identicon_image,
        cert_width as i64 - 140,
        50,
    );

    // Draw title
    let title = "Certificate of Achievement";
    draw_text_centered(
        &mut cert_image,
        &TITLE_FONT,
        TITLE_FONT_SIZE,
        title,
        120,
        HIGHLIGHT_COLOR,
    );

    // Draw achievement message
    let achievement = format!(
        "Successfully completed the {} path",
        certificate_data.game_path_name
    );
    draw_text_centered(
        &mut cert_image,
        &BODY_FONT,
        BODY_FONT_SIZE,
        &achievement,
        300,
        TEXT_COLOR,
    );

    // Draw performance summary
    let performance = format!(
        "Completed {} out of {} challenges with {}% performance",
        certificate_data.solved_challenges,
        certificate_data.total_challenges,
        certificate_data.performance_percentage
    );
    draw_text_centered(
        &mut cert_image,
        &BODY_FONT,
        BODY_FONT_SIZE,
        &performance,
        350,
        TEXT_COLOR,
    );

    // Draw date
    let date_str = format!("Issued on {}", certificate_data.date.format("%d %B %Y"));
    draw_text_centered(
        &mut cert_image,
        &BODY_FONT,
        SMALL_FONT_SIZE,
        &date_str,
        460,
        TEXT_COLOR,
    );

    // Draw issuer
    let issued_by_message = format!("Issued by {}", issuer);
    draw_text_centered(
        &mut cert_image,
        &BODY_FONT,
        SMALL_FONT_SIZE,
        &issued_by_message,
        510,
        HIGHLIGHT_COLOR,
    );

    // Draw QR code
    let qr_code_size = resized_qr_code_image.width() as u32;
    imageops::overlay(
        &mut cert_image,
        &resized_qr_code_image,
        (cert_width - qr_code_size) as i64 / 2,
        (cert_height - qr_code_size - 80) as i64, // Moved up slightly
    );

    Ok(DynamicImage::ImageRgba8(cert_image))
}

pub fn calculate_text_width(font: &FontRef, scale: PxScale, text: &str) -> u32 {
    let scaled_font = font.as_scaled(scale);
    text.chars().fold(0.0, |acc, c| {
        acc + scaled_font.h_advance(scaled_font.scaled_glyph(c).id)
    }) as u32
}

pub fn create_certificate_data_url(
    certificate_data: &CertificateData,
    url: &str,
    issuer: &str,
) -> Result<String> {
    let image = create_certificate(certificate_data, url, issuer)?;

    let mut image_data: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut image_data);

    image.write_to(&mut cursor, ImageFormat::Png).map_err(|e| {
        CertificateError::ImageProcessingError(format!("Failed to write image to buffer: {}", e))
    })?;

    let res_base64 = general_purpose::STANDARD.encode(image_data);
    Ok(format!("data:image/png;base64,{}", res_base64))
}

pub fn draw_text_centered(
    image: &mut RgbaImage,
    font: &FontRef,
    font_size: f32,
    text: &str,
    y: u32,
    color: Rgba<u8>,
) {
    let scale = PxScale::from(font_size);
    let text_width = calculate_text_width(font, scale, text);
    let x = (image.width() as i32 - text_width as i32) / 2;
    draw_text_mut(image, color, x, y as i32, scale, font, text);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_create_certificate() {
        let certificate_data = CertificateData::new(
            "Test Game Path".to_string(),
            10,
            5,
            "Test Player".to_string(),
            Utc::now(),
        );
        let url = "https://example.com".to_string();
        let issuer = "Test Issuer";

        let image = create_certificate(&certificate_data, &url, issuer).unwrap();

        assert!(image.width() > 100);
        assert!(image.height() > 100);
    }

    #[test]
    fn test_create_certificate_data_url() {
        let certificate_data = CertificateData::new(
            "Test Game Path".to_string(),
            10,
            5,
            "Test Player".to_string(),
            Utc::now(),
        );
        let url = "https://example.com";
        let issuer = "Test Issuer";

        let data_url = create_certificate_data_url(&certificate_data, url, issuer).unwrap();

        assert!(data_url.starts_with("data:image/png;base64,"));
    }

    #[test]
    fn test_error_handling_invalid_url() {
        // Test with an invalid URL that would cause QR code generation to fail
        let certificate_data = CertificateData::new(
            "Test Game Path".to_string(),
            10,
            5,
            "Test Player".to_string(),
            Utc::now(),
        );

        // Create an extremely long URL that exceeds QR code capacity for EcLevel::H
        // The maximum capacity for QR code with error correction level H is around 1273 characters
        // for version 40, so we'll use something longer than that
        let url = "a".repeat(2000); // This should be too long for a QR code
        let issuer = "Test Issuer";

        let result = create_certificate(&certificate_data, &url, issuer);
        assert!(result.is_err());

        // Verify the error type
        match result {
            Err(CertificateError::ImageProcessingError(_)) => {} // Expected error
            Err(e) => panic!("Unexpected error type: {:?}", e),
            Ok(_) => panic!("Expected an error but got success"),
        }
    }
}
