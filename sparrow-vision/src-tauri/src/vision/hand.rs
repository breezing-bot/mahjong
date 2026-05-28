use crate::types::{Detection, RecognitionLayout, RecognitionMeld, RecognitionMeldKind};
use std::cmp::Ordering;

pub fn infer_layout(tiles: &[Detection]) -> RecognitionLayout {
  if tiles.is_empty() {
    return empty_layout();
  }

  let median_height = median(tiles.iter().map(|tile| tile.bbox.height).collect());
  let median_width = median(tiles.iter().map(|tile| tile.bbox.width).collect());
  let main_row_ids = infer_main_row_ids(tiles, median_height);
  let mut main_row = tiles_by_ids(tiles, &main_row_ids);
  main_row.sort_by(compare_left);

  let hora = infer_hora(&main_row, median_width);
  let hand = main_row
    .iter()
    .filter(|tile| Some(tile.id) != hora)
    .map(|tile| tile.id)
    .collect();

  let remaining: Vec<&Detection> = tiles
    .iter()
    .filter(|tile| !main_row_ids.iter().any(|id| *id == tile.id))
    .collect();
  let (naki, unassigned) = infer_naki(remaining, median_width, median_height);

  RecognitionLayout {
    hand,
    hora,
    naki,
    unassigned,
  }
}

fn empty_layout() -> RecognitionLayout {
  RecognitionLayout {
    hand: Vec::new(),
    hora: None,
    naki: Vec::new(),
    unassigned: Vec::new(),
  }
}

fn infer_main_row_ids(tiles: &[Detection], median_height: f32) -> Vec<u32> {
  let max_bottom = tiles
    .iter()
    .map(|tile| tile.bbox.y + tile.bbox.height)
    .fold(0.0_f32, f32::max);
  let threshold = max_bottom - median_height.max(24.0) * 1.35;

  let mut row: Vec<&Detection> = tiles
    .iter()
    .filter(|tile| tile.bbox.y + tile.bbox.height >= threshold)
    .collect();
  row.sort_by(compare_left);
  row.into_iter().map(|tile| tile.id).collect()
}

fn infer_hora(main_row: &[&Detection], median_width: f32) -> Option<u32> {
  if main_row.is_empty() {
    return None;
  }
  if main_row.len() == 1 {
    return Some(main_row[0].id);
  }

  let mut largest_gap = f32::NEG_INFINITY;
  let mut largest_gap_index = 0;
  for index in 0..main_row.len() - 1 {
    let left = main_row[index];
    let right = main_row[index + 1];
    let gap = right.bbox.x - (left.bbox.x + left.bbox.width);
    if gap > largest_gap {
      largest_gap = gap;
      largest_gap_index = index;
    }
  }

  if largest_gap >= median_width.max(1.0) * 0.75 && largest_gap_index == main_row.len() - 2 {
    return Some(main_row[main_row.len() - 1].id);
  }

  Some(main_row[main_row.len() - 1].id)
}

fn infer_naki(tiles: Vec<&Detection>, median_width: f32, median_height: f32) -> (Vec<RecognitionMeld>, Vec<u32>) {
  if tiles.is_empty() {
    return (Vec::new(), Vec::new());
  }

  let mut remaining = tiles;
  remaining.sort_by(|a, b| {
    center_y(a)
      .partial_cmp(&center_y(b))
      .unwrap_or(Ordering::Equal)
      .then_with(|| compare_left(a, b))
  });

  let mut rows: Vec<Vec<&Detection>> = Vec::new();
  for tile in remaining {
    if let Some(row) = rows.iter_mut().find(|row| (center_y(row[0]) - center_y(tile)).abs() <= median_height * 0.9) {
      row.push(tile);
    } else {
      rows.push(vec![tile]);
    }
  }

  let mut naki = Vec::new();
  let mut unassigned = Vec::new();

  for mut row in rows {
    row.sort_by(compare_left);
    let mut current: Vec<&Detection> = Vec::new();
    for tile in row {
      let starts_new_group = current.last().is_some_and(|last| {
        tile.bbox.x - (last.bbox.x + last.bbox.width) > median_width.max(1.0) * 1.25
      });
      if starts_new_group {
        push_meld_or_unassigned(&mut naki, &mut unassigned, current);
        current = Vec::new();
      }
      current.push(tile);
    }
    push_meld_or_unassigned(&mut naki, &mut unassigned, current);
  }

  (naki, unassigned)
}

fn push_meld_or_unassigned(naki: &mut Vec<RecognitionMeld>, unassigned: &mut Vec<u32>, group: Vec<&Detection>) {
  if group.len() == 3 || group.len() == 4 {
    let (kind, needs_confirmation) = infer_meld_kind(&group);
    naki.push(RecognitionMeld {
      id: format!("naki_{}", naki.len() + 1),
      kind,
      tiles: group.iter().map(|tile| tile.id).collect(),
      needs_confirmation,
    });
  } else {
    unassigned.extend(group.into_iter().map(|tile| tile.id));
  }
}

fn infer_meld_kind(group: &[&Detection]) -> (RecognitionMeldKind, bool) {
  if group.len() == 4 && same_tile(group) {
    return (RecognitionMeldKind::Daiminkan, false);
  }
  if group.len() == 3 && same_tile(group) {
    return (RecognitionMeldKind::Pon, false);
  }
  if group.len() == 3 && is_sequence(group) {
    return (RecognitionMeldKind::Chi, false);
  }
  (RecognitionMeldKind::Unknown, true)
}

fn same_tile(group: &[&Detection]) -> bool {
  group
    .first()
    .map(|first| group.iter().all(|tile| normalized_tile(&tile.tile_id) == normalized_tile(&first.tile_id)))
    .unwrap_or(false)
}

fn is_sequence(group: &[&Detection]) -> bool {
  let suit = group[0].tile_id.chars().last();
  if !matches!(suit, Some('m' | 'p' | 's')) || group.iter().any(|tile| tile.tile_id.chars().last() != suit) {
    return false;
  }

  let mut numbers: Vec<u8> = group.iter().filter_map(|tile| normalized_number(&tile.tile_id)).collect();
  numbers.sort_unstable();
  numbers.len() == 3 && numbers[0] + 1 == numbers[1] && numbers[1] + 1 == numbers[2]
}

fn normalized_tile(tile_id: &str) -> String {
  if tile_id.starts_with('0') {
    format!("5{}", &tile_id[1..])
  } else {
    tile_id.to_string()
  }
}

fn normalized_number(tile_id: &str) -> Option<u8> {
  let number = tile_id.chars().next()?.to_digit(10)? as u8;
  Some(if number == 0 { 5 } else { number })
}

fn tiles_by_ids<'a>(tiles: &'a [Detection], ids: &[u32]) -> Vec<&'a Detection> {
  ids.iter()
    .filter_map(|id| tiles.iter().find(|tile| tile.id == *id))
    .collect()
}

fn center_y(tile: &Detection) -> f32 {
  tile.bbox.y + tile.bbox.height / 2.0
}

fn compare_left(a: &&Detection, b: &&Detection) -> Ordering {
  a.bbox.x.partial_cmp(&b.bbox.x).unwrap_or(Ordering::Equal)
}

fn median(mut values: Vec<f32>) -> f32 {
  if values.is_empty() {
    return 0.0;
  }
  values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
  values[values.len() / 2]
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::types::BBox;

  fn tile(id: u32, tile_id: &str, x: f32, y: f32) -> Detection {
    Detection {
      id,
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
  fn bottom_row_becomes_hand_with_spaced_hora() {
    let tiles = vec![
      tile(1, "1m", 10.0, 300.0),
      tile(2, "2m", 45.0, 300.0),
      tile(3, "3m", 110.0, 300.0),
      tile(4, "9p", 10.0, 100.0),
    ];

    let layout = infer_layout(&tiles);

    assert_eq!(layout.hand, vec![1, 2]);
    assert_eq!(layout.hora, Some(3));
  }

  #[test]
  fn rightmost_tile_becomes_hora_without_gap() {
    let tiles = vec![
      tile(1, "1m", 10.0, 300.0),
      tile(2, "2m", 45.0, 300.0),
      tile(3, "3m", 80.0, 300.0),
    ];

    let layout = infer_layout(&tiles);

    assert_eq!(layout.hand, vec![1, 2]);
    assert_eq!(layout.hora, Some(3));
  }

  #[test]
  fn groups_non_main_row_tiles_into_melds() {
    let tiles = vec![
      tile(1, "1m", 10.0, 300.0),
      tile(2, "2m", 45.0, 300.0),
      tile(3, "3m", 80.0, 300.0),
      tile(4, "4p", 10.0, 100.0),
      tile(5, "5p", 45.0, 100.0),
      tile(6, "6p", 80.0, 100.0),
      tile(7, "7z", 150.0, 100.0),
      tile(8, "7z", 185.0, 100.0),
      tile(9, "7z", 220.0, 100.0),
      tile(10, "7z", 255.0, 100.0),
    ];

    let layout = infer_layout(&tiles);

    assert_eq!(layout.naki.len(), 2);
    assert!(matches!(layout.naki[0].kind, RecognitionMeldKind::Chi));
    assert!(matches!(layout.naki[1].kind, RecognitionMeldKind::Daiminkan));
  }

  #[test]
  fn leaves_irregular_group_unassigned() {
    let tiles = vec![
      tile(1, "1m", 10.0, 300.0),
      tile(2, "2m", 45.0, 300.0),
      tile(3, "3m", 80.0, 300.0),
      tile(4, "4p", 10.0, 100.0),
      tile(5, "5p", 45.0, 100.0),
    ];

    let layout = infer_layout(&tiles);

    assert_eq!(layout.unassigned, vec![4, 5]);
  }
}
