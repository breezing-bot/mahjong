mod hand;
mod model;
mod preprocess;
mod recognizer;
mod yolo;

use crate::types::RecognitionResult;
use tauri::AppHandle;

const INPUT_SIZE: u32 = 1280;
const CONFIDENCE_THRESHOLD: f32 = 0.25;

pub fn recognize_image(image_bytes: &[u8], app: &AppHandle) -> Result<RecognitionResult, String> {
  recognizer::recognize_image(image_bytes, app)
}
