pub fn normalize_closed_hand(
  hand_tiles: &[String], winning_tile: &str, meld_count: usize,
) -> Result<Vec<String>, String> {
  let expected = 13usize.saturating_sub(meld_count * 3);
  if hand_tiles.len() == expected {
    return Ok(hand_tiles.to_vec());
  }
  if hand_tiles.len() == expected + 1 {
    let mut normalized = hand_tiles.to_vec();
    if let Some(index) = normalized.iter().position(|tile| tile == winning_tile) {
      normalized.remove(index);
      return Ok(normalized);
    }
  }

  Err(format!(
    "手牌数量不正确：当前 {} 张，副露 {} 组时应为 {} 张（不含和了牌）",
    hand_tiles.len(),
    meld_count,
    expected
  ))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn tiles(count: usize) -> Vec<String> {
    (0..count).map(|_| "m1".to_string()).collect()
  }

  #[test]
  fn accepts_expected_closed_hand_count() {
    assert_eq!(normalize_closed_hand(&tiles(13), "m1", 0).unwrap().len(), 13);
    assert_eq!(normalize_closed_hand(&tiles(10), "m1", 1).unwrap().len(), 10);
  }

  #[test]
  fn removes_winning_tile_when_hand_contains_it() {
    let mut hand = tiles(13);
    hand.push("p1".to_string());

    let normalized = normalize_closed_hand(&hand, "p1", 0).unwrap();

    assert_eq!(normalized.len(), 13);
    assert!(!normalized.contains(&"p1".to_string()));
  }

  #[test]
  fn rejects_wrong_count_or_missing_winning_tile() {
    assert!(normalize_closed_hand(&tiles(12), "m1", 0).is_err());

    let mut hand = tiles(13);
    hand.push("p1".to_string());
    assert!(normalize_closed_hand(&hand, "s1", 0).is_err());
  }
}
