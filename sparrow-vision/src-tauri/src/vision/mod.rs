mod hand;
mod model;
mod nms;
mod preprocess;
mod yolo;

use crate::types::{RecognitionDiagnostics, RecognitionResult};
use tauri::AppHandle;
use tract_onnx::prelude::*;

const INPUT_SIZE: u32 = 1280;
const CONFIDENCE_THRESHOLD: f32 = 0.25;
const NMS_IOU_THRESHOLD: f32 = 0.45;
const MAX_DETECTIONS: usize = 32;

pub fn recognize_image(image_bytes: &[u8], app: &AppHandle) -> RecognitionResult {
    match recognize_image_inner(image_bytes, app) {
        Ok(result) => result,
        Err(_) => unavailable_result(),
    }
}

fn recognize_image_inner(image_bytes: &[u8], app: &AppHandle) -> Result<RecognitionResult, String> {
    let image =
        image::load_from_memory(image_bytes).map_err(|err| format!("图片解码失败：{err}"))?;
    let (input, letterbox) = preprocess::prepare_input(image);
    let model = model::cached_model(app, INPUT_SIZE)?;

    let tensor = Tensor::from_shape(&[1, 3, INPUT_SIZE as usize, INPUT_SIZE as usize], &input)
        .map_err(|err| err.to_string())?;
    let outputs = model
        .run(tvec!(tensor.into()))
        .map_err(|err| err.to_string())?;
    let output = outputs[0]
        .to_array_view::<f32>()
        .map_err(|err| err.to_string())?;

    let (raw, mut diagnostics) = yolo::parse_output(&output, &letterbox, CONFIDENCE_THRESHOLD);
    let nms = nms::suppress_overlaps(raw, NMS_IOU_THRESHOLD, MAX_DETECTIONS, &mut diagnostics);
    let hand_tiles = hand::infer_hand_tiles(&nms.detections);
    let quality_flags = quality_flags(&diagnostics, nms.trimmed_to_max);

    Ok(RecognitionResult {
        detections: nms.detections,
        hand_tiles,
        quality_flags,
        diagnostics,
    })
}

fn quality_flags(diagnostics: &RecognitionDiagnostics, trimmed_to_max: bool) -> Vec<String> {
    let mut flags = Vec::new();
    if diagnostics.dropped_low_confidence_count > 0 {
        flags.push("low_confidence_filtered".to_string());
    }
    if diagnostics.dropped_unknown_class_count > 0 {
        flags.push("unknown_class_index".to_string());
    }
    if diagnostics.dropped_duplicate_count > 0 {
        flags.push("duplicate_tile_suppressed".to_string());
    }
    if trimmed_to_max {
        flags.push("trimmed_to_max_detections".to_string());
    }
    if diagnostics.kept_detection_count == 0 {
        flags.push("empty_model_output".to_string());
    }
    flags
}

fn unavailable_result() -> RecognitionResult {
    RecognitionResult {
        detections: Vec::new(),
        hand_tiles: Vec::new(),
        quality_flags: vec!["model_unavailable".to_string()],
        diagnostics: RecognitionDiagnostics {
            raw_detection_count: 0,
            kept_detection_count: 0,
            dropped_low_confidence_count: 0,
            dropped_unknown_class_count: 0,
            dropped_duplicate_count: 0,
        },
    }
}
