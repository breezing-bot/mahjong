use crate::types::{BBox, Detection, RecognitionDiagnostics, RecognitionResultV1};
use image::{imageops::FilterType, DynamicImage, ImageBuffer, Rgb};
use std::cmp::Ordering;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tauri::{path::BaseDirectory, AppHandle, Manager};
use tract_onnx::prelude::*;

const INPUT_SIZE: u32 = 1280;
const CONFIDENCE_THRESHOLD: f32 = 0.25;
const NMS_IOU_THRESHOLD: f32 = 0.45;
const MAX_DETECTIONS: usize = 32;

const TILE_VOCAB: [&str; 45] = [
    "m5r", "m1", "m2", "m3", "m4", "m5", "m6", "m7", "m8", "m9", "p5r", "p1", "p2", "p3", "p4",
    "p5", "p6", "p7", "p8", "p9", "s5r", "s1", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9",
    "z1", "z2", "z3", "z4", "z5", "z6", "z7", "f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8",
];

type VisionModel = Arc<TypedRunnableModel<TypedModel>>;
static MODEL: OnceLock<VisionModel> = OnceLock::new();

#[derive(Debug, Clone)]
struct RawDetection {
    tile_id: String,
    confidence: f32,
    bbox: BBox,
}

#[derive(Debug, Clone)]
struct LetterboxMeta {
    original_width: f32,
    original_height: f32,
    scale: f32,
    pad_x: f32,
    pad_y: f32,
}

pub fn recognize_image(image_bytes: &[u8], app: &AppHandle) -> RecognitionResultV1 {
    match recognize_image_inner(image_bytes, app) {
        Ok(result) => result,
        Err(_) => unavailable_result(),
    }
}

fn recognize_image_inner(
    image_bytes: &[u8],
    app: &AppHandle,
) -> Result<RecognitionResultV1, String> {
    let image = image::load_from_memory(image_bytes)
        .map_err(|err| format!("image decode failed: {err}"))?;
    let (input, meta) = preprocess(image);
    let model = model(app)?;

    let tensor = Tensor::from_shape(&[1, 3, INPUT_SIZE as usize, INPUT_SIZE as usize], &input)
        .map_err(|err| err.to_string())?;
    let outputs = model
        .run(tvec!(tensor.into()))
        .map_err(|err| err.to_string())?;
    let output = outputs[0]
        .to_array_view::<f32>()
        .map_err(|err| err.to_string())?;
    let (raw, mut diagnostics) = parse_yolo_output(&output, &meta);
    let detections = apply_nms(raw, &mut diagnostics);
    let hand_tiles = infer_hand_tiles(&detections);
    let mut quality_flags = quality_flags(&diagnostics);

    if detections.is_empty() {
        quality_flags.push("empty_model_output".to_string());
    }

    Ok(RecognitionResultV1 {
        detections,
        hand_tiles,
        quality_flags,
        diagnostics,
    })
}

fn model(app: &AppHandle) -> Result<VisionModel, String> {
    if let Some(model) = MODEL.get() {
        return Ok(Arc::clone(model));
    }

    let loaded = load_model(app)?;
    if MODEL.set(Arc::clone(&loaded)).is_err() {
        return MODEL
            .get()
            .map(Arc::clone)
            .ok_or_else(|| "model cache initialization failed".to_string());
    }
    Ok(loaded)
}

fn load_model(app: &AppHandle) -> Result<VisionModel, String> {
    let model_path = model_path(app);
    let model = tract_onnx::onnx()
        .model_for_path(model_path)
        .map_err(|err| err.to_string())?
        .with_input_fact(
            0,
            f32::fact([1, 3, INPUT_SIZE as usize, INPUT_SIZE as usize]).into(),
        )
        .map_err(|err| err.to_string())?
        .into_optimized()
        .map_err(|err| err.to_string())?
        .into_runnable()
        .map_err(|err| err.to_string())?;
    Ok(Arc::new(model))
}

fn preprocess(image: DynamicImage) -> (Vec<f32>, LetterboxMeta) {
    let rgb = image.to_rgb8();
    let (original_width, original_height) = rgb.dimensions();
    let scale =
        (INPUT_SIZE as f32 / original_width as f32).min(INPUT_SIZE as f32 / original_height as f32);
    let resized_width = (original_width as f32 * scale).round() as u32;
    let resized_height = (original_height as f32 * scale).round() as u32;
    let resized =
        image::imageops::resize(&rgb, resized_width, resized_height, FilterType::Triangle);
    let mut canvas = ImageBuffer::from_pixel(INPUT_SIZE, INPUT_SIZE, Rgb([114, 114, 114]));
    let pad_x = (INPUT_SIZE - resized_width) / 2;
    let pad_y = (INPUT_SIZE - resized_height) / 2;
    image::imageops::replace(&mut canvas, &resized, pad_x.into(), pad_y.into());

    let plane_size = (INPUT_SIZE * INPUT_SIZE) as usize;
    let mut input = vec![0.0; plane_size * 3];
    for (x, y, pixel) in canvas.enumerate_pixels() {
        let offset = (y * INPUT_SIZE + x) as usize;
        input[offset] = pixel[0] as f32 / 255.0;
        input[plane_size + offset] = pixel[1] as f32 / 255.0;
        input[plane_size * 2 + offset] = pixel[2] as f32 / 255.0;
    }

    (
        input,
        LetterboxMeta {
            original_width: original_width as f32,
            original_height: original_height as f32,
            scale,
            pad_x: pad_x as f32,
            pad_y: pad_y as f32,
        },
    )
}

fn parse_yolo_output(
    output: &tract_ndarray::ArrayViewD<f32>,
    meta: &LetterboxMeta,
) -> (Vec<RawDetection>, RecognitionDiagnostics) {
    let shape = output.shape();
    let mut candidates = Vec::new();
    let mut diagnostics = RecognitionDiagnostics {
        raw_detection_count: 0,
        kept_detection_count: 0,
        dropped_low_confidence_count: 0,
        dropped_unknown_class_count: 0,
        dropped_duplicate_count: 0,
    };

    if shape.len() != 3 {
        return (candidates, diagnostics);
    }

    let rows_are_predictions = shape[2] == TILE_VOCAB.len() + 4;
    let prediction_count = if rows_are_predictions {
        shape[1]
    } else {
        shape[2]
    };
    diagnostics.raw_detection_count = prediction_count;

    for index in 0..prediction_count {
        let mut values = Vec::with_capacity(TILE_VOCAB.len() + 4);
        if rows_are_predictions {
            for j in 0..shape[2] {
                values.push(output[[0, index, j]]);
            }
        } else {
            for j in 0..shape[1] {
                values.push(output[[0, j, index]]);
            }
        }

        if values.len() < TILE_VOCAB.len() + 4 {
            continue;
        }

        let (class_index, confidence) = values[4..]
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(Ordering::Equal))
            .map(|(idx, value)| (idx, *value))
            .unwrap_or((usize::MAX, 0.0));

        if confidence < CONFIDENCE_THRESHOLD {
            diagnostics.dropped_low_confidence_count += 1;
            continue;
        }
        let Some(tile_id) = TILE_VOCAB.get(class_index) else {
            diagnostics.dropped_unknown_class_count += 1;
            continue;
        };

        candidates.push(RawDetection {
            tile_id: tile_id.to_string(),
            confidence,
            bbox: remap_bbox(values[0], values[1], values[2], values[3], meta),
        });
    }

    (candidates, diagnostics)
}

fn remap_bbox(cx: f32, cy: f32, width: f32, height: f32, meta: &LetterboxMeta) -> BBox {
    let x1 = ((cx - width / 2.0) - meta.pad_x) / meta.scale;
    let y1 = ((cy - height / 2.0) - meta.pad_y) / meta.scale;
    let x2 = ((cx + width / 2.0) - meta.pad_x) / meta.scale;
    let y2 = ((cy + height / 2.0) - meta.pad_y) / meta.scale;
    let x1 = x1.clamp(0.0, meta.original_width);
    let y1 = y1.clamp(0.0, meta.original_height);
    let x2 = x2.clamp(0.0, meta.original_width);
    let y2 = y2.clamp(0.0, meta.original_height);

    BBox {
        x: x1,
        y: y1,
        width: (x2 - x1).max(0.0),
        height: (y2 - y1).max(0.0),
    }
}

fn apply_nms(
    mut raw: Vec<RawDetection>,
    diagnostics: &mut RecognitionDiagnostics,
) -> Vec<Detection> {
    raw.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(Ordering::Equal)
    });

    let mut kept: Vec<RawDetection> = Vec::new();
    for candidate in raw {
        if kept
            .iter()
            .any(|existing| iou(&candidate.bbox, &existing.bbox) > NMS_IOU_THRESHOLD)
        {
            diagnostics.dropped_duplicate_count += 1;
            continue;
        }
        kept.push(candidate);
        if kept.len() == MAX_DETECTIONS {
            break;
        }
    }

    diagnostics.kept_detection_count = kept.len();
    kept.into_iter()
        .map(|raw| Detection {
            tile_id: raw.tile_id,
            confidence: raw.confidence,
            bbox: raw.bbox,
        })
        .collect()
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

fn infer_hand_tiles(detections: &[Detection]) -> Vec<String> {
    if detections.is_empty() {
        return Vec::new();
    }

    let max_bottom = detections
        .iter()
        .map(|d| d.bbox.y + d.bbox.height)
        .fold(0.0_f32, f32::max);
    let avg_height =
        detections.iter().map(|d| d.bbox.height).sum::<f32>() / detections.len() as f32;
    let threshold = max_bottom - avg_height.max(24.0) * 1.5;
    let mut bottom: Vec<&Detection> = detections
        .iter()
        .filter(|d| d.bbox.y + d.bbox.height >= threshold)
        .collect();
    bottom.sort_by(|a, b| a.bbox.x.partial_cmp(&b.bbox.x).unwrap_or(Ordering::Equal));
    bottom.into_iter().map(|d| d.tile_id.clone()).collect()
}

fn quality_flags(diagnostics: &RecognitionDiagnostics) -> Vec<String> {
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
    flags
}

fn unavailable_result() -> RecognitionResultV1 {
    RecognitionResultV1 {
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

fn model_path(app: &AppHandle) -> PathBuf {
    let resource_path = app
        .path()
        .resolve("model/last.onnx", BaseDirectory::Resource);
    if let Ok(path) = resource_path {
        if path.exists() {
            return path;
        }
    }

    source_model_path()
}

fn source_model_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("ml")
        .join("model")
        .join("last.onnx")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nms_removes_overlapping_boxes() {
        let mut diagnostics = RecognitionDiagnostics {
            raw_detection_count: 2,
            kept_detection_count: 0,
            dropped_low_confidence_count: 0,
            dropped_unknown_class_count: 0,
            dropped_duplicate_count: 0,
        };
        let detections = apply_nms(
            vec![
                RawDetection {
                    tile_id: "m1".to_string(),
                    confidence: 0.9,
                    bbox: BBox {
                        x: 0.0,
                        y: 0.0,
                        width: 20.0,
                        height: 20.0,
                    },
                },
                RawDetection {
                    tile_id: "m2".to_string(),
                    confidence: 0.8,
                    bbox: BBox {
                        x: 1.0,
                        y: 1.0,
                        width: 20.0,
                        height: 20.0,
                    },
                },
            ],
            &mut diagnostics,
        );
        assert_eq!(detections.len(), 1);
        assert_eq!(diagnostics.dropped_duplicate_count, 1);
    }

    #[test]
    fn bottom_row_becomes_hand_tiles() {
        let detections = vec![
            Detection {
                tile_id: "m1".to_string(),
                confidence: 0.9,
                bbox: BBox {
                    x: 20.0,
                    y: 400.0,
                    width: 30.0,
                    height: 40.0,
                },
            },
            Detection {
                tile_id: "m2".to_string(),
                confidence: 0.9,
                bbox: BBox {
                    x: 60.0,
                    y: 400.0,
                    width: 30.0,
                    height: 40.0,
                },
            },
            Detection {
                tile_id: "p1".to_string(),
                confidence: 0.9,
                bbox: BBox {
                    x: 20.0,
                    y: 100.0,
                    width: 30.0,
                    height: 40.0,
                },
            },
        ];

        assert_eq!(infer_hand_tiles(&detections), vec!["m1", "m2"]);
    }
}
