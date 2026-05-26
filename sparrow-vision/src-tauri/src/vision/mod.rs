mod hand;
mod model;
mod nms;
mod output;
mod preprocess;
mod recognizer;
mod yolo;

use crate::types::RecognitionResult;
use tauri::AppHandle;

const INPUT_SIZE: u32 = 1280;
const CONFIDENCE_THRESHOLD: f32 = 0.25;
const NMS_IOU_THRESHOLD: f32 = 0.45;
const MAX_DETECTIONS: usize = 32;

pub fn recognize_image(image_bytes: &[u8], app: &AppHandle) -> RecognitionResult {
  recognizer::recognize_image_or_unavailable(image_bytes, app)
}
