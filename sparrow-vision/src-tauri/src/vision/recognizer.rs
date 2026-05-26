use super::{hand, model, nms, output, preprocess, yolo};
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

  let (raw, mut diagnostics) = yolo::parse_output(&output, &letterbox, super::CONFIDENCE_THRESHOLD);
  let nms = nms::suppress_overlaps(raw, super::NMS_IOU_THRESHOLD, super::MAX_DETECTIONS, &mut diagnostics);
  let hand_tiles = hand::infer_hand_tiles(&nms.detections);

  Ok(output::recognition_result(
    nms.detections,
    hand_tiles,
    diagnostics,
    nms.trimmed_to_max,
  ))
}

pub fn recognize_image_or_unavailable(image_bytes: &[u8], app: &AppHandle) -> RecognitionResult {
  recognize_image(image_bytes, app).unwrap_or_else(|_| output::unavailable_result())
}

#[cfg(test)]
mod debug_tests {
  use super::*;
  use std::env;
  use std::fs;
  use std::path::{Path, PathBuf};

  #[test]
  #[ignore = "set SPARROW_VISION_DEBUG_IMAGE to an image path and run with --ignored --nocapture"]
  fn debug_model_output_for_image() {
    let Some(image_path) = debug_image_path() else {
      print_missing_image_help();
      return;
    };

    let image_bytes =
      fs::read(&image_path).unwrap_or_else(|err| panic!("failed to read {}: {err}", image_path.display()));
    let image = image::load_from_memory(&image_bytes)
      .unwrap_or_else(|err| panic!("failed to decode {}: {err}", image_path.display()));
    let (input, letterbox) = preprocess::prepare_input(image);
    let tensor = Tensor::from_shape(
      &[
        1,
        3,
        super::super::INPUT_SIZE as usize,
        super::super::INPUT_SIZE as usize,
      ],
      &input,
    )
    .unwrap();
    let model_path = model::source_model_path();
    let model = model::load_model_from_path(model_path.clone(), super::super::INPUT_SIZE)
      .unwrap_or_else(|err| panic!("failed to load {}: {err}", model_path.display()));
    let outputs = model.run(tvec!(tensor.into())).unwrap();
    let output = outputs[0].to_array_view::<f32>().unwrap();

    println!("image_path={}", image_path.display());
    println!("model_path={}", model_path.display());
    println!(
      "letterbox={{ original_width: {}, original_height: {}, scale: {}, pad_x: {}, pad_y: {} }}",
      letterbox.original_width, letterbox.original_height, letterbox.scale, letterbox.pad_x, letterbox.pad_y
    );
    println!("input_len={}", input.len());
    println!("model_output_shape={:?}", output.shape());
    println!("model_output_stats={:?}", stats(output.iter().copied()));
    println!(
      "model_output_first_values={:?}",
      output.iter().take(24).copied().collect::<Vec<_>>()
    );
  }

  #[test]
  #[ignore = "set SPARROW_VISION_DEBUG_IMAGE to an image path and run with --ignored --nocapture"]
  fn debug_output_to_recognition_result_for_image() {
    let Some(image_path) = debug_image_path() else {
      print_missing_image_help();
      return;
    };

    let image_bytes =
      fs::read(&image_path).unwrap_or_else(|err| panic!("failed to read {}: {err}", image_path.display()));
    let image = image::load_from_memory(&image_bytes)
      .unwrap_or_else(|err| panic!("failed to decode {}: {err}", image_path.display()));
    let (input, letterbox) = preprocess::prepare_input(image);
    let tensor = Tensor::from_shape(
      &[
        1,
        3,
        super::super::INPUT_SIZE as usize,
        super::super::INPUT_SIZE as usize,
      ],
      &input,
    )
    .unwrap();
    let model_path = model::source_model_path();
    let model = model::load_model_from_path(model_path.clone(), super::super::INPUT_SIZE)
      .unwrap_or_else(|err| panic!("failed to load {}: {err}", model_path.display()));
    let outputs = model.run(tvec!(tensor.into())).unwrap();
    let model_output = outputs[0].to_array_view::<f32>().unwrap();

    let (raw, mut diagnostics) = yolo::parse_output(&model_output, &letterbox, super::super::CONFIDENCE_THRESHOLD);
    println!("raw_detection_count={}", raw.len());
    println!("raw_detections_sample={:#?}", raw.iter().take(16).collect::<Vec<_>>());
    println!("diagnostics_after_yolo={:#?}", diagnostics);

    let nms = nms::suppress_overlaps(
      raw,
      super::super::NMS_IOU_THRESHOLD,
      super::super::MAX_DETECTIONS,
      &mut diagnostics,
    );
    println!("detections_after_nms={:#?}", nms.detections);
    println!("diagnostics_after_nms={:#?}", diagnostics);
    println!("trimmed_to_max={}", nms.trimmed_to_max);

    let hand_tiles = hand::infer_hand_tiles(&nms.detections);
    println!("hand_tiles={:#?}", hand_tiles);

    let result = output::recognition_result(nms.detections, hand_tiles, diagnostics, nms.trimmed_to_max);
    println!(
      "recognition_result_json={}",
      serde_json::to_string_pretty(&result).unwrap()
    );
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
    println!("  cargo test -p sparrow-vision debug_output_to_recognition_result_for_image -- --ignored --nocapture");
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
