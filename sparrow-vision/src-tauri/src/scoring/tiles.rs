use riichi_calc::constants::tiles::{Tile, TileType};
use std::str::FromStr;

pub fn parse_tile_id(tile_id: &str) -> Result<Tile, String> {
  if tile_id.ends_with('f') {
    return Err(format!("{tile_id} 是花牌，v1 日麻计分不接受花牌"));
  }

  match tile_id {
    "0m" => Ok(Tile {
      number: 10,
      tile_type: TileType::Manzu,
    }),
    "0p" => Ok(Tile {
      number: 10,
      tile_type: TileType::Pinzu,
    }),
    "0s" => Ok(Tile {
      number: 10,
      tile_type: TileType::Souzu,
    }),
    _ => Tile::from_str(tile_id).map_err(|err| format!("未知牌 ID：{tile_id} ({err:?})")),
  }
}

pub fn normalize_red(tile: Tile) -> Tile {
  if tile.number == 10 {
    Tile {
      number: 5,
      tile_type: tile.tile_type,
    }
  } else {
    tile
  }
}

pub fn dora_from_indicator(tile_id: &str) -> Result<Tile, String> {
  let tile = normalize_red(parse_tile_id(tile_id)?);
  let number = match tile.tile_type {
    TileType::Manzu | TileType::Pinzu | TileType::Souzu => {
      if tile.number == 9 {
        1
      } else {
        tile.number + 1
      }
    }
    TileType::Wind => {
      if tile.number == 4 {
        1
      } else {
        tile.number + 1
      }
    }
    TileType::Dragon => match tile.number {
      1 => 2,
      2 => 3,
      3 => 1,
      _ => return Err(format!("非法三元牌指示牌：{tile_id}")),
    },
  };
  Ok(Tile {
    number,
    tile_type: tile.tile_type,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_number_tiles_and_red_fives() {
    let tile = parse_tile_id("0m").unwrap();
    assert_eq!(tile.number, 10);
    assert_eq!(tile.tile_type, TileType::Manzu);

    let tile = parse_tile_id("9p").unwrap();
    assert_eq!(tile.number, 9);
    assert_eq!(tile.tile_type, TileType::Pinzu);
  }

  #[test]
  fn parses_honor_tiles() {
    assert_eq!(parse_tile_id("1z").unwrap().tile_type, TileType::Wind);
    assert_eq!(parse_tile_id("5z").unwrap().tile_type, TileType::Dragon);
    assert_eq!(parse_tile_id("5z").unwrap().number, 1);
  }

  #[test]
  fn rejects_flower_and_invalid_tiles() {
    assert!(parse_tile_id("1f").is_err());
    assert!(parse_tile_id("x1").is_err());
    assert!(parse_tile_id("m0").is_err());
    assert!(parse_tile_id("8z").is_err());
  }

  #[test]
  fn converts_dora_indicators() {
    assert_eq!(dora_from_indicator("9m").unwrap().number, 1);
    assert_eq!(dora_from_indicator("4z").unwrap().number, 1);
    assert_eq!(dora_from_indicator("5z").unwrap().number, 2);
    assert_eq!(dora_from_indicator("6z").unwrap().number, 3);
    assert_eq!(dora_from_indicator("7z").unwrap().number, 1);
    assert_eq!(dora_from_indicator("0m").unwrap().number, 6);
  }
}
