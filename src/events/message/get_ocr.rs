use super::MessageError;
use leptess::LepTess;
use std::io::Cursor;

pub async fn handler(url: &String) -> Result<String, MessageError> {
    let request = reqwest::get(url)
        .await
        .map_err(|_| MessageError::HttpRequestError)?;

    if request
        .headers()
        .get("Content-Type")
        .expect("Failed to get content type")
        .to_str()
        .map_err(|_| MessageError::HttpRequestError)?
        .contains("image/jpeg")
        || request
            .headers()
            .get("Content-Type")
            .expect("Failed to get content type")
            .to_str()
            .map_err(|_| MessageError::HttpRequestError)?
            .contains("image/png")
    {
        let image = image::load_from_memory(
            &request
                .bytes()
                .await
                .map_err(|_| MessageError::ImageLoadError)?,
        )
        .map_err(|_| MessageError::ImageLoadError)?;
        let mut tiff_buff = Vec::new();

        image
            .write_to(
                &mut Cursor::new(&mut tiff_buff),
                image::ImageOutputFormat::Tiff,
            )
            .map_err(|_| MessageError::ImageLoadError)?;

        let mut tesseract =
            LepTess::new(Some("./tessdata"), "eng").map_err(|_| MessageError::TesseractError)?;

        tesseract
            .set_image_from_mem(&tiff_buff)
            .expect("Failed to set image from memory");
        tesseract.set_source_resolution(70);

        return Ok(tesseract
            .get_utf8_text()
            .map_err(|_| MessageError::TesseractError)?);
    }

    Err(MessageError::BadType)
}
