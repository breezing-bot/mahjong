use crate::rules;
use crate::types::{AnalyzeHandRequestV1, RecognitionResultV1, ScoringResultV1};
use crate::vision;

#[tauri::command]
pub fn recognize_image(
    app: tauri::AppHandle,
    image_bytes: Vec<u8>,
) -> Result<RecognitionResultV1, String> {
    Ok(vision::recognize_image(&image_bytes, &app))
}

#[tauri::command]
pub fn analyze_hand(request: AnalyzeHandRequestV1) -> Result<ScoringResultV1, String> {
    Ok(rules::analyze_hand(request))
}
