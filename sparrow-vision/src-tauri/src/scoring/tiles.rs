use riichi_calc::constants::tiles::{Tile, TileType};

pub fn parse_tile_id(tile_id: &str) -> Result<Tile, String> {
  if tile_id.starts_with('f') {
    return Err(format!("{tile_id} 是花牌，v1 日麻计分不接受花牌"));
  }

  let bytes = tile_id.as_bytes();
  if bytes.len() < 2 {
    return Err(format!("未知牌 ID：{tile_id}"));
  }

  let tile_type = match bytes[0] as char {
    'm' => TileType::Manzu,
    'p' => TileType::Pinzu,
    's' => TileType::Souzu,
    'z' => return parse_honor(tile_id),
    _ => return Err(format!("未知牌 ID：{tile_id}")),
  };

  let number = if tile_id.ends_with('r') {
    10
  } else {
    tile_id[1..].parse::<u8>().map_err(|_| format!("未知数牌：{tile_id}"))?
  };
  if !(1..=10).contains(&number) {
    return Err(format!("未知数牌：{tile_id}"));
  }

  Ok(Tile { number, tile_type })
}

fn parse_honor(tile_id: &str) -> Result<Tile, String> {
  let number = tile_id[1..].parse::<u8>().map_err(|_| format!("未知字牌：{tile_id}"))?;
  match number {
    1..=4 => Ok(Tile {
      number,
      tile_type: TileType::Wind,
    }),
    5..=7 => Ok(Tile {
      number: number - 4,
      tile_type: TileType::Dragon,
    }),
    _ => Err(format!("未知字牌：{tile_id}")),
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
  fn maps_number_tiles_and_red_fives() {
    let tile = parse_tile_id("m5r").unwrap();
    assert_eq!(tile.number, 10);
    assert_eq!(tile.tile_type, TileType::Manzu);

    let tile = parse_tile_id("p9").unwrap();
    assert_eq!(tile.number, 9);
    assert_eq!(tile.tile_type, TileType::Pinzu);
  }

  #[test]
  fn maps_honor_tiles() {
    assert_eq!(parse_tile_id("z1").unwrap().tile_type, TileType::Wind);
    assert_eq!(parse_tile_id("z5").unwrap().tile_type, TileType::Dragon);
    assert_eq!(parse_tile_id("z5").unwrap().number, 1);
  }

  #[test]
  fn rejects_flower_and_invalid_tiles() {
    assert!(parse_tile_id("f1").is_err());
    assert!(parse_tile_id("x1").is_err());
    assert!(parse_tile_id("m0").is_err());
    assert!(parse_tile_id("z8").is_err());
  }

  #[test]
  fn converts_dora_indicators() {
    assert_eq!(dora_from_indicator("m9").unwrap().number, 1);
    assert_eq!(dora_from_indicator("z4").unwrap().number, 1);
    assert_eq!(dora_from_indicator("z5").unwrap().number, 2);
    assert_eq!(dora_from_indicator("z6").unwrap().number, 3);
    assert_eq!(dora_from_indicator("z7").unwrap().number, 1);
    assert_eq!(dora_from_indicator("m5r").unwrap().number, 6);
  }
}
