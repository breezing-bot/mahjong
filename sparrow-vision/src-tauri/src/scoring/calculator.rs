use super::{melds, yaku};
use crate::types::{
  AnalyzeHandRequest, RiichiInput, ScoringResult, SpecialWinInput, TsumoPoints, WinMethodInput, WindInput,
};
use riichi_calc::calculator::result::Points;
use riichi_calc::constants::field::{Field, Wind};
use riichi_calc::constants::status::{RiichiStatus, SpecialWin, Status, WinMethod};
use riichi_calc::constants::tiles::Tile;
use riichi_calc::parser::{Input, PiInput};
use std::collections::HashSet;
use std::str::FromStr;

pub fn analyze_hand(request: AnalyzeHandRequest) -> ScoringResult {
  match analyze_hand_inner(request) {
    Ok(result) => result,
    Err(error) => ScoringResult {
      is_win: false,
      yaku: Vec::new(),
      han: 0,
      fu: 0,
      ron_points: None,
      tsumo_points: None,
      waits: Vec::new(),
      errors: vec![error],
    },
  }
}

fn analyze_hand_inner(request: AnalyzeHandRequest) -> Result<ScoringResult, String> {
  let hand = parse_tiles(&request.hand)?;
  let naki = request
    .naki
    .iter()
    .map(melds::convert_meld)
    .collect::<Result<Vec<_>, _>>()?;
  let dora = request
    .dora
    .iter()
    .map(|tile| {
      Tile::from_str(tile)
        .map(Tile::dora_from_indicator)
        .map_err(|err| format!("{err:?}"))
    })
    .collect::<Result<Vec<_>, _>>()?;
  let ura_dora = request
    .ura_dora
    .iter()
    .map(|tile| {
      Tile::from_str(tile)
        .map(Tile::dora_from_indicator)
        .map_err(|err| format!("{err:?}"))
    })
    .collect::<Result<Vec<_>, _>>()?;

  let output = Input::new(
    PiInput {
      hand,
      naki,
      hora: Tile::from_str(&request.hora).map_err(|err| format!("{err:?}"))?,
    },
    Field {
      zikaze: wind(&request.zikaze),
      bakaze: wind(&request.bakaze),
      honba: request.honba,
      dora,
    },
    Status {
      riichi: riichi(&request.riichi, ura_dora),
      win_method: win_method(&request.win_method),
      special_win: special_wins(&request.special_win),
    },
  )
  .calc_hand()
  .map_err(|err| format!("输入不是合法和牌形：{err:?}"))?;

  let (ron_points, tsumo_points) = display_points(&output.score_result.actual_points);
  Ok(ScoringResult {
    is_win: true,
    yaku: yaku::flatten_yaku(&output.found_result),
    han: output.score_result.detail.han,
    fu: output.score_result.detail.fu,
    ron_points,
    tsumo_points,
    waits: Vec::new(),
    errors: Vec::new(),
  })
}

fn parse_tiles(tile_ids: &[String]) -> Result<Vec<Tile>, String> {
  tile_ids
    .iter()
    .map(|tile| Tile::from_str(tile).map_err(|err| format!("{err:?}")))
    .collect()
}

fn display_points(points: &Points) -> (Option<u32>, Option<TsumoPoints>) {
  match points {
    Points::Ron(value) => (Some(*value), None),
    Points::ChildTumo(non_dealer, dealer) => (
      None,
      Some(TsumoPoints {
        dealer: *dealer,
        non_dealer: *non_dealer,
      }),
    ),
    Points::DealerTumo(value) => (
      None,
      Some(TsumoPoints {
        dealer: *value,
        non_dealer: *value,
      }),
    ),
  }
}

fn wind(wind: &WindInput) -> Wind {
  match wind {
    WindInput::East => Wind::East,
    WindInput::South => Wind::South,
    WindInput::West => Wind::West,
    WindInput::North => Wind::North,
  }
}

fn win_method(method: &WinMethodInput) -> WinMethod {
  match method {
    WinMethodInput::Ron => WinMethod::Ron,
    WinMethodInput::Tsumo => WinMethod::Tsumo,
  }
}

fn riichi(riichi: &RiichiInput, ura_dora: Vec<Tile>) -> RiichiStatus {
  match riichi {
    RiichiInput::None => RiichiStatus::NoRiichi,
    RiichiInput::Riichi => RiichiStatus::Riichi(ura_dora),
    RiichiInput::DoubleRiichi => RiichiStatus::DoubleRiichi(ura_dora),
  }
}

fn special_wins(input: &SpecialWinInput) -> HashSet<SpecialWin> {
  let mut wins = HashSet::new();
  if input.ippatsu {
    wins.insert(SpecialWin::Ipatu);
  }
  if input.chankan {
    wins.insert(SpecialWin::Chankan);
  }
  if input.rinshan {
    wins.insert(SpecialWin::Rinshan);
  }
  if input.haitei {
    wins.insert(SpecialWin::Haitei);
  }
  if input.hotei {
    wins.insert(SpecialWin::Hotei);
  }
  if input.first_turn_tsumo {
    wins.insert(SpecialWin::DaiichiTumo);
  }
  wins
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::types::{MeldInput, MeldKind};

  fn base_request() -> AnalyzeHandRequest {
    AnalyzeHandRequest {
      hand: vec![
        "1m", "2m", "3m", "5m", "6m", "7m", "2p", "3p", "4p", "6s", "7s", "9s", "9s",
      ]
      .into_iter()
      .map(String::from)
      .collect(),
      hora: "5s".to_string(),
      naki: Vec::new(),
      zikaze: WindInput::East,
      bakaze: WindInput::East,
      win_method: WinMethodInput::Ron,
      riichi: RiichiInput::Riichi,
      special_win: SpecialWinInput::default(),
      honba: 0,
      dora: Vec::new(),
      ura_dora: Vec::new(),
    }
  }

  #[test]
  fn scores_simple_pinfu_riichi_ron() {
    let result = analyze_hand(base_request());

    assert!(result.is_win, "{:?}", result.errors);
    assert!(result.han >= 1);
    assert!(result.fu >= 20);
    assert!(result.ron_points.is_some());
  }

  #[test]
  fn scores_closed_tsumo_hand() {
    let mut request = base_request();
    request.win_method = WinMethodInput::Tsumo;

    let result = analyze_hand(request);

    assert!(result.is_win, "{:?}", result.errors);
    assert!(result.tsumo_points.is_some());
  }

  #[test]
  fn lets_calc_hand_report_wrong_closed_tile_count() {
    let mut request = base_request();
    request.hand.pop();

    let result = analyze_hand(request);

    assert!(!result.is_win);
    assert!(result.errors[0].contains("HandValidationError"));
  }

  #[test]
  fn accepts_open_meld_shape() {
    let request = AnalyzeHandRequest {
      hand: vec!["4m", "5m", "6m", "2p", "3p", "4p", "6s", "7s", "9s", "9s"]
        .into_iter()
        .map(String::from)
        .collect(),
      hora: "5s".to_string(),
      naki: vec![MeldInput {
        kind: MeldKind::Pon,
        tiles: vec!["5z", "5z", "5z"].into_iter().map(String::from).collect(),
      }],
      zikaze: WindInput::East,
      bakaze: WindInput::East,
      win_method: WinMethodInput::Ron,
      riichi: RiichiInput::None,
      special_win: SpecialWinInput::default(),
      honba: 0,
      dora: Vec::new(),
      ura_dora: Vec::new(),
    };

    let result = analyze_hand(request);

    assert!(result.is_win, "{:?}", result.errors);
  }

  #[test]
  fn converts_dora_indicators_with_riichi_calc_tile_api() {
    let mut request = base_request();
    request.dora = vec!["9m".to_string(), "4z".to_string(), "7z".to_string()];

    let result = analyze_hand(request);

    assert!(result.is_win, "{:?}", result.errors);
  }
}
