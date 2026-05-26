use crate::types::{Detection, RecognitionDiagnostics, RecognitionResult};

pub fn recognition_result(
  detections: Vec<Detection>, hand_tiles: Vec<String>, diagnostics: RecognitionDiagnostics, trimmed_to_max: bool,
) -> RecognitionResult {
  let quality_flags = quality_flags(&diagnostics, trimmed_to_max);
  RecognitionResult {
    detections,
    hand_tiles,
    quality_flags,
    diagnostics,
  }
}

pub fn unavailable_result() -> RecognitionResult {
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
