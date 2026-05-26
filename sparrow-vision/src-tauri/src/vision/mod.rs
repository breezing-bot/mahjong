mod hand;
mod model;
mod output;
mod preprocess;
mod recognizer;
mod yolo;

use crate::types::RecognitionResult;
use tauri::AppHandle;

const INPUT_SIZE: u32 = 1280;
const CONFIDENCE_THRESHOLD: f32 = 0.25;

pub fn recognize_image(image_bytes: &[u8], app: &AppHandle) -> RecognitionResult {
  recognizer::recognize_image_or_unavailable(image_bytes, app)
}
