use crate::types::{Detection, RecognitionLayout, RecognitionResult};

pub fn recognition_result(detections: Vec<Detection>, layout: RecognitionLayout) -> RecognitionResult {
  RecognitionResult {
    detections,
    layout,
  }
}

pub fn unavailable_result() -> RecognitionResult {
  RecognitionResult {
    detections: Vec::new(),
    layout: RecognitionLayout {
      hand: Vec::new(),
      hora: None,
      naki: Vec::new(),
      unassigned: Vec::new(),
    },
  }
}
