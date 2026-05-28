pub fn normalize_closed_hand(hand_tiles: &[String], meld_count: usize) -> Result<Vec<String>, String> {
  let expected = 13usize.saturating_sub(meld_count * 3);
  if hand_tiles.len() == expected {
    return Ok(hand_tiles.to_vec());
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
    (0..count).map(|_| "1m".to_string()).collect()
  }

  #[test]
  fn accepts_expected_closed_hand_count() {
    assert_eq!(normalize_closed_hand(&tiles(13), 0).unwrap().len(), 13);
    assert_eq!(normalize_closed_hand(&tiles(10), 1).unwrap().len(), 10);
  }

  #[test]
  fn rejects_wrong_count() {
    assert!(normalize_closed_hand(&tiles(12), 0).is_err());
    assert!(normalize_closed_hand(&tiles(14), 0).is_err());
  }
}
