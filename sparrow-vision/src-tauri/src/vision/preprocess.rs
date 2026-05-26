use image::{imageops::FilterType, DynamicImage, ImageBuffer, Rgb};

#[derive(Debug, Clone)]
pub struct Letterbox {
  pub original_width: f32,
  pub original_height: f32,
  pub scale: f32,
  pub pad_x: f32,
  pub pad_y: f32,
}

pub fn prepare_input(image: DynamicImage) -> (Vec<f32>, Letterbox) {
  let rgb = image.to_rgb8();
  let (original_width, original_height) = rgb.dimensions();
  let input_size = super::INPUT_SIZE;
  let scale = (input_size as f32 / original_width as f32).min(input_size as f32 / original_height as f32);
  let resized_width = (original_width as f32 * scale).round() as u32;
  let resized_height = (original_height as f32 * scale).round() as u32;
  let resized = image::imageops::resize(&rgb, resized_width, resized_height, FilterType::Triangle);
  let mut canvas = ImageBuffer::from_pixel(input_size, input_size, Rgb([114, 114, 114]));
  let pad_x = (input_size - resized_width) / 2;
  let pad_y = (input_size - resized_height) / 2;
  image::imageops::replace(&mut canvas, &resized, pad_x.into(), pad_y.into());

  let plane_size = (input_size * input_size) as usize;
  let mut input = vec![0.0; plane_size * 3];
  for (x, y, pixel) in canvas.enumerate_pixels() {
    let offset = (y * input_size + x) as usize;
    input[offset] = pixel[0] as f32 / 255.0;
    input[plane_size + offset] = pixel[1] as f32 / 255.0;
    input[plane_size * 2 + offset] = pixel[2] as f32 / 255.0;
  }

  (
    input,
    Letterbox {
      original_width: original_width as f32,
      original_height: original_height as f32,
      scale,
      pad_x: pad_x as f32,
      pad_y: pad_y as f32,
    },
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use image::{DynamicImage, RgbImage};

  #[test]
  fn records_letterbox_padding_for_wide_image() {
    let image = DynamicImage::ImageRgb8(RgbImage::new(640, 320));
    let (_, letterbox) = prepare_input(image);

    assert_eq!(letterbox.original_width, 640.0);
    assert_eq!(letterbox.original_height, 320.0);
    assert_eq!(letterbox.scale, 2.0);
    assert_eq!(letterbox.pad_x, 0.0);
    assert_eq!(letterbox.pad_y, 320.0);
  }
}
