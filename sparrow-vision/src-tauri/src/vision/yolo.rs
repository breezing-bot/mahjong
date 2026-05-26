use super::preprocess::Letterbox;
use crate::tile_vocab::TILE_IDS;
use crate::types::{BBox, RecognitionDiagnostics};
use std::cmp::Ordering;
use tract_onnx::prelude::tract_ndarray::ArrayViewD;

#[derive(Debug, Clone)]
pub struct RawDetection {
    pub tile_id: String,
    pub confidence: f32,
    pub bbox: BBox,
}

pub fn parse_output(
    output: &ArrayViewD<f32>,
    letterbox: &Letterbox,
    confidence_threshold: f32,
) -> (Vec<RawDetection>, RecognitionDiagnostics) {
    let shape = output.shape();
    let mut candidates = Vec::new();
    let mut diagnostics = empty_diagnostics();

    if shape.len() != 3 {
        return (candidates, diagnostics);
    }

    let Some(layout) = OutputLayout::from_shape(shape) else {
        return (candidates, diagnostics);
    };
    let prediction_count = layout.prediction_count(shape);
    diagnostics.raw_detection_count = prediction_count;

    for index in 0..prediction_count {
        let values = prediction_values(output, index, layout);
        let (class_index, confidence) = best_class(&values);
        if confidence < confidence_threshold {
            diagnostics.dropped_low_confidence_count += 1;
            continue;
        }

        let Some(tile_id) = TILE_IDS.get(class_index) else {
            diagnostics.dropped_unknown_class_count += 1;
            continue;
        };

        candidates.push(RawDetection {
            tile_id: tile_id.to_string(),
            confidence,
            bbox: remap_bbox(values[0], values[1], values[2], values[3], letterbox),
        });
    }

    (candidates, diagnostics)
}

#[derive(Debug, Clone, Copy)]
enum OutputLayout {
    PredictionsByRow,
    ValuesByRow,
}

impl OutputLayout {
    fn from_shape(shape: &[usize]) -> Option<Self> {
        let value_count = TILE_IDS.len() + 4;
        if shape[2] == value_count {
            Some(Self::PredictionsByRow)
        } else if shape[1] == value_count {
            Some(Self::ValuesByRow)
        } else {
            None
        }
    }

    fn prediction_count(self, shape: &[usize]) -> usize {
        match self {
            Self::PredictionsByRow => shape[1],
            Self::ValuesByRow => shape[2],
        }
    }

    fn value_count(self, shape: &[usize]) -> usize {
        match self {
            Self::PredictionsByRow => shape[2],
            Self::ValuesByRow => shape[1],
        }
    }
}

fn prediction_values(output: &ArrayViewD<f32>, index: usize, layout: OutputLayout) -> Vec<f32> {
    let shape = output.shape();
    let value_count = layout.value_count(shape);
    let mut values = Vec::with_capacity(value_count);
    for value_index in 0..value_count {
        let value = match layout {
            OutputLayout::PredictionsByRow => output[[0, index, value_index]],
            OutputLayout::ValuesByRow => output[[0, value_index, index]],
        };
        values.push(value);
    }
    values
}

fn best_class(values: &[f32]) -> (usize, f32) {
    values[4..]
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(Ordering::Equal))
        .map(|(idx, value)| (idx, *value))
        .unwrap_or((usize::MAX, 0.0))
}

fn remap_bbox(cx: f32, cy: f32, width: f32, height: f32, letterbox: &Letterbox) -> BBox {
    let x1 = ((cx - width / 2.0) - letterbox.pad_x) / letterbox.scale;
    let y1 = ((cy - height / 2.0) - letterbox.pad_y) / letterbox.scale;
    let x2 = ((cx + width / 2.0) - letterbox.pad_x) / letterbox.scale;
    let y2 = ((cy + height / 2.0) - letterbox.pad_y) / letterbox.scale;
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
        dropped_duplicate_count: 0,
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
        let bbox = remap_bbox(320.0, 640.0, 64.0, 80.0, &letterbox());

        assert_eq!(bbox.x, 144.0);
        assert_eq!(bbox.y, 140.0);
        assert_eq!(bbox.width, 32.0);
        assert_eq!(bbox.height, 40.0);
    }

    #[test]
    fn parses_prediction_rows_layout() {
        let mut output = Array::zeros((1, 1, TILE_IDS.len() + 4));
        output[[0, 0, 0]] = 320.0;
        output[[0, 0, 1]] = 640.0;
        output[[0, 0, 2]] = 64.0;
        output[[0, 0, 3]] = 80.0;
        output[[0, 0, 5]] = 0.9;

        let (detections, diagnostics) = parse_output(&output.into_dyn().view(), &letterbox(), 0.25);

        assert_eq!(diagnostics.raw_detection_count, 1);
        assert_eq!(detections.len(), 1);
        assert_eq!(detections[0].tile_id, "m1");
    }

    #[test]
    fn parses_channel_rows_layout() {
        let mut output = Array::zeros((1, TILE_IDS.len() + 4, 1));
        output[[0, 0, 0]] = 320.0;
        output[[0, 1, 0]] = 640.0;
        output[[0, 2, 0]] = 64.0;
        output[[0, 3, 0]] = 80.0;
        output[[0, 6, 0]] = 0.9;

        let (detections, diagnostics) = parse_output(&output.into_dyn().view(), &letterbox(), 0.25);

        assert_eq!(diagnostics.raw_detection_count, 1);
        assert_eq!(detections.len(), 1);
        assert_eq!(detections[0].tile_id, "m2");
    }

    #[test]
    fn parses_many_channel_first_predictions() {
        let mut output = Array::zeros((1, TILE_IDS.len() + 4, 8400));
        output[[0, 0, 99]] = 320.0;
        output[[0, 1, 99]] = 640.0;
        output[[0, 2, 99]] = 64.0;
        output[[0, 3, 99]] = 80.0;
        output[[0, 7, 99]] = 0.9;

        let (detections, diagnostics) = parse_output(&output.into_dyn().view(), &letterbox(), 0.25);

        assert_eq!(diagnostics.raw_detection_count, 8400);
        assert_eq!(detections.len(), 1);
        assert_eq!(detections[0].tile_id, "m3");
    }

    #[test]
    fn counts_low_confidence_predictions() {
        let mut output = Array::zeros((1, 2, TILE_IDS.len() + 4));
        output[[0, 0, 4]] = 0.1;
        output[[0, 1, 4]] = 0.2;

        let (detections, diagnostics) = parse_output(&output.into_dyn().view(), &letterbox(), 0.25);

        assert!(detections.is_empty());
        assert_eq!(diagnostics.dropped_low_confidence_count, 2);
        assert_eq!(diagnostics.dropped_unknown_class_count, 0);
    }
}
