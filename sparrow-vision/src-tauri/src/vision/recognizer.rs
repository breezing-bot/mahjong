use super::{hand, model, output, preprocess, yolo};
use crate::types::RecognitionResult;
use tauri::AppHandle;
use tract_onnx::prelude::*;

pub fn recognize_image(image_bytes: &[u8], app: &AppHandle) -> Result<RecognitionResult, String> {
  let image = image::load_from_memory(image_bytes).map_err(|err| format!("图片解码失败：{err}"))?;
  let (input, letterbox) = preprocess::prepare_input(image);
  let model = model::cached_model(app, super::INPUT_SIZE)?;

  let tensor = Tensor::from_shape(&[1, 3, super::INPUT_SIZE as usize, super::INPUT_SIZE as usize], &input)
    .map_err(|err| err.to_string())?;
  let outputs = model.run(tvec!(tensor.into())).map_err(|err| err.to_string())?;
  let output = outputs[0].to_array_view::<f32>().map_err(|err| err.to_string())?;

  let detections = yolo::parse_output(&output, &letterbox, super::CONFIDENCE_THRESHOLD);
  let layout = hand::infer_layout(&detections);

  Ok(output::recognition_result(detections, layout))
}

pub fn recognize_image_or_unavailable(image_bytes: &[u8], app: &AppHandle) -> RecognitionResult {
  recognize_image(image_bytes, app).unwrap_or_else(|_| output::unavailable_result())
}

#[cfg(test)]
mod debug_tests {
  use super::*;
  use crate::types::{BBox, Detection};
  use image::{Rgb, RgbImage};
  use std::env;
  use std::fs;
  use std::path::{Path, PathBuf};
  use tract_onnx::prelude::tract_ndarray::{ArrayD, ArrayViewD};

  #[test]
  #[ignore = "set SPARROW_VISION_DEBUG_IMAGE to an image path and run with --ignored --nocapture"]
  fn debug_model_output_for_image() {
    let Some(image_path) = debug_image_path() else {
      print_missing_image_help();
      return;
    };

    let (_, input, letterbox, model_output) = run_model_for_debug(&image_path);

    println!("image_path={}", image_path.display());
    println!("letterbox={letterbox:?}");
    println!("input_len={}", input.len());
    println!("model_output_shape={:?}", model_output.shape());
    println!("model_output_stats={:?}", stats(model_output.iter().copied()));
    println!(
      "model_output_first_values={:?}",
      model_output.iter().take(24).copied().collect::<Vec<_>>()
    );
    print_detection_rows(&model_output.view());
  }

  #[test]
  #[ignore = "set SPARROW_VISION_DEBUG_IMAGE to an image path and run with --ignored --nocapture"]
  fn debug_output_to_recognition_result_for_image() {
    let Some(image_path) = debug_image_path() else {
      print_missing_image_help();
      return;
    };

    let (_, _, letterbox, model_output) = run_model_for_debug(&image_path);
    print_detection_rows(&model_output.view());

    let detections = yolo::parse_output(&model_output.view(), &letterbox, super::super::CONFIDENCE_THRESHOLD);
    println!("detection_count={}", detections.len());
    println!(
      "detections_sample={:#?}",
      detections.iter().take(16).collect::<Vec<_>>()
    );

    let layout = hand::infer_layout(&detections);
    println!("layout={layout:#?}");

    let result = output::recognition_result(detections, layout);
    println!(
      "recognition_result_json={}",
      serde_json::to_string_pretty(&result).unwrap()
    );
  }

  #[test]
  #[ignore = "set SPARROW_VISION_DEBUG_IMAGE to an image path and run with --ignored --nocapture"]
  fn debug_save_annotated_recognition_image() {
    let Some(image_path) = debug_image_path() else {
      print_missing_image_help();
      return;
    };

    let (image_bytes, _, letterbox, model_output) = run_model_for_debug(&image_path);
    let detections = yolo::parse_output(&model_output.view(), &letterbox, super::super::CONFIDENCE_THRESHOLD);
    let layout = hand::infer_layout(&detections);
    let result = output::recognition_result(detections, layout);

    let mut annotated = image::load_from_memory(&image_bytes)
      .unwrap_or_else(|err| panic!("failed to decode {}: {err}", image_path.display()))
      .to_rgb8();
    for detection in &result.detections {
      draw_bbox(&mut annotated, &detection.bbox, color_for_detection(detection));
    }

    let output_path = annotated_output_path(&image_path);
    annotated
      .save(&output_path)
      .unwrap_or_else(|err| panic!("failed to save {}: {err}", output_path.display()));

    println!("image_path={}", image_path.display());
    println!("annotated_output_path={}", output_path.display());
    println!("detection_count={}", result.detections.len());
    println!("layout={:#?}", result.layout);
  }

  fn run_model_for_debug(image_path: &Path) -> (Vec<u8>, Vec<f32>, preprocess::Letterbox, ArrayD<f32>) {
    let image_bytes =
      fs::read(image_path).unwrap_or_else(|err| panic!("failed to read {}: {err}", image_path.display()));
    let image = image::load_from_memory(&image_bytes)
      .unwrap_or_else(|err| panic!("failed to decode {}: {err}", image_path.display()));
    let (input, letterbox) = preprocess::prepare_input(image);
    let tensor = input_tensor(&input);
    let model_path = model::source_model_path();
    let model = model::load_model_from_path(model_path.clone(), super::super::INPUT_SIZE)
      .unwrap_or_else(|err| panic!("failed to load {}: {err}", model_path.display()));
    let outputs = model.run(tvec!(tensor.into())).unwrap();
    let model_output = outputs[0].to_array_view::<f32>().unwrap().to_owned().into_dyn();

    (image_bytes, input, letterbox, model_output)
  }

  fn input_tensor(input: &[f32]) -> Tensor {
    Tensor::from_shape(
      &[
        1,
        3,
        super::super::INPUT_SIZE as usize,
        super::super::INPUT_SIZE as usize,
      ],
      input,
    )
    .unwrap()
  }

  fn draw_bbox(image: &mut RgbImage, bbox: &BBox, color: Rgb<u8>) {
    let max_x = image.width().saturating_sub(1) as i32;
    let max_y = image.height().saturating_sub(1) as i32;
    let x1 = bbox.x.round().clamp(0.0, max_x as f32) as i32;
    let y1 = bbox.y.round().clamp(0.0, max_y as f32) as i32;
    let x2 = (bbox.x + bbox.width).round().clamp(0.0, max_x as f32) as i32;
    let y2 = (bbox.y + bbox.height).round().clamp(0.0, max_y as f32) as i32;

    for inset in 0..3 {
      draw_rect_outline(image, x1 + inset, y1 + inset, x2 - inset, y2 - inset, color);
    }
  }

  fn draw_rect_outline(image: &mut RgbImage, x1: i32, y1: i32, x2: i32, y2: i32, color: Rgb<u8>) {
    if x1 > x2 || y1 > y2 {
      return;
    }

    for x in x1..=x2 {
      put_pixel_checked(image, x, y1, color);
      put_pixel_checked(image, x, y2, color);
    }
    for y in y1..=y2 {
      put_pixel_checked(image, x1, y, color);
      put_pixel_checked(image, x2, y, color);
    }
  }

  fn put_pixel_checked(image: &mut RgbImage, x: i32, y: i32, color: Rgb<u8>) {
    if x >= 0 && y >= 0 && x < image.width() as i32 && y < image.height() as i32 {
      image.put_pixel(x as u32, y as u32, color);
    }
  }

  fn color_for_detection(detection: &Detection) -> Rgb<u8> {
    if detection.confidence >= 0.75 {
      Rgb([0, 255, 80])
    } else if detection.confidence >= 0.5 {
      Rgb([255, 220, 0])
    } else {
      Rgb([255, 64, 64])
    }
  }

  fn annotated_output_path(image_path: &Path) -> PathBuf {
    let stem = image_path
      .file_stem()
      .and_then(|stem| stem.to_str())
      .unwrap_or("recognition");
    env::temp_dir().join(format!("{stem}.annotated.jpg"))
  }

  fn print_detection_rows(output: &ArrayViewD<f32>) {
    if output.shape().len() != 3 || output.shape()[2] != 6 {
      return;
    }

    let row_count = output.shape()[1].min(16);
    println!("model_output_detection_rows_sample=[x1, y1, x2, y2, conf, cls]");
    for index in 0..row_count {
      println!(
        "  row[{index}] = [{}, {}, {}, {}, {}, {}]",
        output[[0, index, 0]],
        output[[0, index, 1]],
        output[[0, index, 2]],
        output[[0, index, 3]],
        output[[0, index, 4]],
        output[[0, index, 5]]
      );
    }
  }

  fn debug_image_path() -> Option<PathBuf> {
    env::var_os("SPARROW_VISION_DEBUG_IMAGE")
      .map(PathBuf::from)
      .filter(|path| path.exists())
      .or_else(first_fixture_image)
  }

  fn first_fixture_image() -> Option<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
      .join("tests")
      .join("fixtures")
      .join("recognition")
      .join("images");
    let entries = fs::read_dir(root).ok()?;
    entries.filter_map(Result::ok).map(|entry| entry.path()).find(|path| {
      path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| matches!(extension.to_ascii_lowercase().as_str(), "jpg" | "jpeg" | "png" | "webp"))
        .unwrap_or(false)
    })
  }

  fn print_missing_image_help() {
    println!("No debug image found.");
    println!("Set SPARROW_VISION_DEBUG_IMAGE to an image path, for example:");
    println!(
            "  $env:SPARROW_VISION_DEBUG_IMAGE='D:\\\\WorkSpace\\\\Bot\\\\mahjong\\\\sparrow-vision\\\\src-tauri\\\\tests\\\\fixtures\\\\recognition\\\\images\\\\sample.jpg'"
        );
    println!("Then run:");
    println!("  cargo test -p sparrow-vision debug_save_annotated_recognition_image -- --ignored --nocapture");
  }

  fn stats(values: impl Iterator<Item = f32>) -> (usize, f32, f32, f32) {
    let mut count = 0;
    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    let mut sum = 0.0;

    for value in values {
      count += 1;
      min = min.min(value);
      max = max.max(value);
      sum += value;
    }

    let mean = if count == 0 { 0.0 } else { sum / count as f32 };
    (count, min, max, mean)
  }
}
