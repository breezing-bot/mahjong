use crate::scoring;
use crate::types::{AnalyzeHandRequest, RecognitionResult, ScoringResult};
use crate::vision;

#[tauri::command]
pub fn recognize_image(app: tauri::AppHandle, image_bytes: Vec<u8>) -> Result<RecognitionResult, String> {
  Ok(vision::recognize_image(&image_bytes, &app))
}

#[tauri::command]
pub fn analyze_hand(request: AnalyzeHandRequest) -> Result<ScoringResult, String> {
  Ok(scoring::analyze_hand(request))
}
