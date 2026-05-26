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
