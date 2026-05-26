use super::yolo::RawDetection;
use crate::types::{BBox, Detection, RecognitionDiagnostics};
use std::cmp::Ordering;

pub struct NmsResult {
    pub detections: Vec<Detection>,
    pub trimmed_to_max: bool,
}

pub fn suppress_overlaps(
    mut raw: Vec<RawDetection>,
    iou_threshold: f32,
    max_detections: usize,
    diagnostics: &mut RecognitionDiagnostics,
) -> NmsResult {
    raw.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(Ordering::Equal)
    });

    let mut kept: Vec<RawDetection> = Vec::new();
    let mut trimmed_to_max = false;
    for candidate in raw {
        if kept
            .iter()
            .any(|existing| iou(&candidate.bbox, &existing.bbox) > iou_threshold)
        {
            diagnostics.dropped_duplicate_count += 1;
            continue;
        }
        if kept.len() == max_detections {
            trimmed_to_max = true;
            break;
        }
        kept.push(candidate);
    }

    diagnostics.kept_detection_count = kept.len();
    NmsResult {
        detections: kept
            .into_iter()
            .map(|raw| Detection {
                tile_id: raw.tile_id,
                confidence: raw.confidence,
                bbox: raw.bbox,
            })
            .collect(),
        trimmed_to_max,
    }
}

fn iou(a: &BBox, b: &BBox) -> f32 {
    let ax2 = a.x + a.width;
    let ay2 = a.y + a.height;
    let bx2 = b.x + b.width;
    let by2 = b.y + b.height;
    let ix1 = a.x.max(b.x);
    let iy1 = a.y.max(b.y);
    let ix2 = ax2.min(bx2);
    let iy2 = ay2.min(by2);
    let intersection = (ix2 - ix1).max(0.0) * (iy2 - iy1).max(0.0);
    let union = a.width * a.height + b.width * b.height - intersection;
    if union <= 0.0 {
        0.0
    } else {
        intersection / union
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diagnostics() -> RecognitionDiagnostics {
        RecognitionDiagnostics {
            raw_detection_count: 0,
            kept_detection_count: 0,
            dropped_low_confidence_count: 0,
            dropped_unknown_class_count: 0,
            dropped_duplicate_count: 0,
        }
    }

    fn raw(tile_id: &str, confidence: f32, x: f32) -> RawDetection {
        RawDetection {
            tile_id: tile_id.to_string(),
            confidence,
            bbox: BBox {
                x,
                y: 0.0,
                width: 20.0,
                height: 20.0,
            },
        }
    }

    #[test]
    fn removes_overlapping_boxes() {
        let mut diagnostics = diagnostics();
        let result = suppress_overlaps(
            vec![raw("m1", 0.9, 0.0), raw("m2", 0.8, 1.0)],
            0.45,
            32,
            &mut diagnostics,
        );

        assert_eq!(result.detections.len(), 1);
        assert_eq!(diagnostics.dropped_duplicate_count, 1);
    }

    #[test]
    fn reports_when_trimmed_to_max_detections() {
        let mut diagnostics = diagnostics();
        let result = suppress_overlaps(
            vec![raw("m1", 0.9, 0.0), raw("m2", 0.8, 30.0)],
            0.45,
            1,
            &mut diagnostics,
        );

        assert_eq!(result.detections.len(), 1);
        assert!(result.trimmed_to_max);
    }
}
