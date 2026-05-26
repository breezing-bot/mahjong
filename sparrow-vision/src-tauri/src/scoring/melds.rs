use super::tiles::{normalize_red, parse_tile_id};
use crate::types::{MeldInput, MeldKind};
use riichi_calc::constants::hand::Mentsu;
use riichi_calc::constants::tiles::{Tile, TileType};

pub fn convert_meld(meld: &MeldInput) -> Result<Mentsu, String> {
  if meld.tiles.is_empty() {
    return Err("副露不能为空".to_string());
  }
  let mut tiles = meld
    .tiles
    .iter()
    .map(|tile| parse_tile_id(tile).map(normalize_red))
    .collect::<Result<Vec<_>, _>>()?;

  match meld.kind {
    MeldKind::Chi => {
      validate_chi(&mut tiles)?;
      Ok(Mentsu::Shuntsu(tiles[0], true))
    }
    MeldKind::Pon => same_tile_mentsu(&tiles, 3).map(|tile| Mentsu::Koutsu(tile, true)),
    MeldKind::Daiminkan => same_tile_mentsu(&tiles, 4).map(|tile| Mentsu::Kantsu(tile, true)),
    MeldKind::Ankan => same_tile_mentsu(&tiles, 4).map(|tile| Mentsu::Kantsu(tile, false)),
  }
}

fn validate_chi(tiles: &mut [Tile]) -> Result<(), String> {
  if tiles.len() != 3 {
    return Err("吃必须正好 3 张牌".to_string());
  }
  tiles.sort_by_key(|tile| tile.number);
  if tiles.iter().any(|tile| tile.tile_type != tiles[0].tile_type)
    || matches!(tiles[0].tile_type, TileType::Wind | TileType::Dragon)
    || tiles[1].number != tiles[0].number + 1
    || tiles[2].number != tiles[1].number + 1
  {
    return Err("吃必须是同花色连续三张数牌".to_string());
  }
  Ok(())
}

fn same_tile_mentsu(tiles: &[Tile], count: usize) -> Result<Tile, String> {
  if tiles.len() != count {
    return Err(format!("该副露必须正好 {count} 张牌"));
  }
  let first = tiles[0];
  if tiles.iter().all(|tile| *tile == first) {
    Ok(first)
  } else {
    Err("碰/杠必须由相同牌组成".to_string())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn meld(kind: MeldKind, tiles: &[&str]) -> MeldInput {
    MeldInput {
      kind,
      tiles: tiles.iter().map(|tile| tile.to_string()).collect(),
    }
  }

  #[test]
  fn converts_valid_melds() {
    assert!(matches!(
      convert_meld(&meld(MeldKind::Chi, &["m1", "m2", "m3"])).unwrap(),
      Mentsu::Shuntsu(_, true)
    ));
    assert!(matches!(
      convert_meld(&meld(MeldKind::Pon, &["z5", "z5", "z5"])).unwrap(),
      Mentsu::Koutsu(_, true)
    ));
    assert!(matches!(
      convert_meld(&meld(MeldKind::Daiminkan, &["p1", "p1", "p1", "p1"])).unwrap(),
      Mentsu::Kantsu(_, true)
    ));
    assert!(matches!(
      convert_meld(&meld(MeldKind::Ankan, &["s9", "s9", "s9", "s9"])).unwrap(),
      Mentsu::Kantsu(_, false)
    ));
  }

  #[test]
  fn rejects_invalid_melds() {
    assert!(convert_meld(&meld(MeldKind::Chi, &["z1", "z2", "z3"])).is_err());
    assert!(convert_meld(&meld(MeldKind::Chi, &["m1", "m2"])).is_err());
    assert!(convert_meld(&meld(MeldKind::Pon, &["m1", "m1", "m2"])).is_err());
    assert!(convert_meld(&meld(MeldKind::Daiminkan, &["m1", "m1", "m1"])).is_err());
  }
}
