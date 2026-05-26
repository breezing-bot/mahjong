use super::preprocess::Letterbox;
use crate::tile_vocab::TILE_IDS;
use crate::types::{BBox, Detection, RecognitionDiagnostics};
use tract_onnx::prelude::tract_ndarray::ArrayViewD;

#[derive(Debug, Clone, Copy)]
struct DetectionRow {
  x1: f32,
  y1: f32,
  x2: f32,
  y2: f32,
  confidence: f32,
  class_id: f32,
}

pub fn parse_output(
  output: &ArrayViewD<f32>, letterbox: &Letterbox, confidence_threshold: f32,
) -> (Vec<Detection>, RecognitionDiagnostics) {
  let shape = output.shape();
  let mut candidates = Vec::new();
  let mut diagnostics = empty_diagnostics();

  if shape.len() != 3 {
    return (candidates, diagnostics);
  }

  if shape[2] != 6 {
    return (candidates, diagnostics);
  }

  let prediction_count = shape[1];
  diagnostics.raw_detection_count = prediction_count;

  for index in 0..prediction_count {
    let Some((class_index, confidence, bbox_values)) = candidate_from_output(output, index) else {
      diagnostics.dropped_unknown_class_count += 1;
      continue;
    };
    if confidence < confidence_threshold {
      diagnostics.dropped_low_confidence_count += 1;
      continue;
    }

    let Some(tile_id) = TILE_IDS.get(class_index) else {
      diagnostics.dropped_unknown_class_count += 1;
      continue;
    };

    candidates.push(Detection {
      tile_id: tile_id.to_string(),
      confidence,
      bbox: remap_corners_bbox(
        bbox_values[0],
        bbox_values[1],
        bbox_values[2],
        bbox_values[3],
        letterbox,
      ),
    });
  }

  (candidates, diagnostics)
}

fn candidate_from_output(output: &ArrayViewD<f32>, index: usize) -> Option<(usize, f32, [f32; 4])> {
  let row = DetectionRow::from_output(output, index);
  if !row.class_id.is_finite() || row.class_id < 0.0 {
    return None;
  }
  Some((
    row.class_id.round() as usize,
    row.confidence,
    [row.x1, row.y1, row.x2, row.y2],
  ))
}

impl DetectionRow {
  fn from_output(output: &ArrayViewD<f32>, index: usize) -> Self {
    Self {
      x1: output[[0, index, 0]],
      y1: output[[0, index, 1]],
      x2: output[[0, index, 2]],
      y2: output[[0, index, 3]],
      confidence: output[[0, index, 4]],
      class_id: output[[0, index, 5]],
    }
  }
}

fn remap_corners_bbox(x1: f32, y1: f32, x2: f32, y2: f32, letterbox: &Letterbox) -> BBox {
  let x1 = (x1 - letterbox.pad_x) / letterbox.scale;
  let y1 = (y1 - letterbox.pad_y) / letterbox.scale;
  let x2 = (x2 - letterbox.pad_x) / letterbox.scale;
  let y2 = (y2 - letterbox.pad_y) / letterbox.scale;
  let x1 = x1.clamp(0.0, letterbox.original_width);
  let y1 = y1.clamp(0.0, letterbox.original_height);
  let x2 = x2.clamp(0.0, letterbox.original_width);
  let y2 = y2.clamp(0.0, letterbox.original_height);

  BBox {
    x: x1,
    y: y1,
    width: (x2 - x1).max(0.0),
    height: (y2 - y1).max(0.0),
  }
}

fn empty_diagnostics() -> RecognitionDiagnostics {
  RecognitionDiagnostics {
    raw_detection_count: 0,
    kept_detection_count: 0,
    dropped_low_confidence_count: 0,
    dropped_unknown_class_count: 0,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tract_onnx::prelude::tract_ndarray::Array;

  fn letterbox() -> Letterbox {
    Letterbox {
      original_width: 640.0,
      original_height: 320.0,
      scale: 2.0,
      pad_x: 0.0,
      pad_y: 320.0,
    }
  }

  #[test]
  fn remaps_bbox_from_letterbox_to_original_pixels() {
    let bbox = remap_corners_bbox(288.0, 600.0, 352.0, 680.0, &letterbox());

    assert_eq!(bbox.x, 144.0);
    assert_eq!(bbox.y, 140.0);
    assert_eq!(bbox.width, 32.0);
    assert_eq!(bbox.height, 40.0);
  }

  #[test]
  fn parses_detection_rows() {
    let mut output = Array::zeros((1, 300, 6));
    output[[0, 7, 0]] = 288.0;
    output[[0, 7, 1]] = 600.0;
    output[[0, 7, 2]] = 352.0;
    output[[0, 7, 3]] = 680.0;
    output[[0, 7, 4]] = 0.9;
    output[[0, 7, 5]] = 3.0;

    let (detections, diagnostics) = parse_output(&output.into_dyn().view(), &letterbox(), 0.25);

    assert_eq!(diagnostics.raw_detection_count, 300);
    assert_eq!(diagnostics.dropped_low_confidence_count, 299);
    assert_eq!(detections.len(), 1);
    assert_eq!(detections[0].tile_id, "m3");
    assert_eq!(detections[0].confidence, 0.9);
    assert_eq!(detections[0].bbox.x, 144.0);
    assert_eq!(detections[0].bbox.y, 140.0);
    assert_eq!(detections[0].bbox.width, 32.0);
    assert_eq!(detections[0].bbox.height, 40.0);
  }

  #[test]
  fn counts_low_confidence_predictions() {
    let mut output = Array::zeros((1, 2, 6));
    output[[0, 0, 4]] = 0.1;
    output[[0, 0, 5]] = 1.0;
    output[[0, 1, 4]] = 0.2;
    output[[0, 1, 5]] = 2.0;

    let (detections, diagnostics) = parse_output(&output.into_dyn().view(), &letterbox(), 0.25);

    assert!(detections.is_empty());
    assert_eq!(diagnostics.dropped_low_confidence_count, 2);
    assert_eq!(diagnostics.dropped_unknown_class_count, 0);
  }

  #[test]
  fn ignores_unsupported_output_shape() {
    let output = Array::zeros((1, TILE_IDS.len() + 4, 8400));

    let (detections, diagnostics) = parse_output(&output.into_dyn().view(), &letterbox(), 0.25);

    assert!(detections.is_empty());
    assert_eq!(diagnostics.raw_detection_count, 0);
  }
}
