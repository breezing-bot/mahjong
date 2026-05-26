use crate::types::Detection;
use std::cmp::Ordering;

pub fn infer_hand_tiles(detections: &[Detection]) -> Vec<String> {
  if detections.is_empty() {
    return Vec::new();
  }

  let max_bottom = detections
    .iter()
    .map(|detection| detection.bbox.y + detection.bbox.height)
    .fold(0.0_f32, f32::max);
  let avg_height = detections.iter().map(|detection| detection.bbox.height).sum::<f32>() / detections.len() as f32;
  let threshold = max_bottom - avg_height.max(24.0) * 1.5;
  let mut bottom: Vec<&Detection> = detections
    .iter()
    .filter(|detection| detection.bbox.y + detection.bbox.height >= threshold)
    .collect();
  bottom.sort_by(|a, b| a.bbox.x.partial_cmp(&b.bbox.x).unwrap_or(Ordering::Equal));
  bottom.into_iter().map(|detection| detection.tile_id.clone()).collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::types::BBox;

  fn detection(tile_id: &str, x: f32, y: f32) -> Detection {
    Detection {
      tile_id: tile_id.to_string(),
      confidence: 0.9,
      bbox: BBox {
        x,
        y,
        width: 30.0,
        height: 40.0,
      },
    }
  }

  #[test]
  fn bottom_row_becomes_hand_tiles_sorted_by_x() {
    let detections = vec![
      detection("m2", 60.0, 400.0),
      detection("p1", 20.0, 100.0),
      detection("m1", 20.0, 400.0),
    ];

    assert_eq!(infer_hand_tiles(&detections), vec!["m1", "m2"]);
  }
}
