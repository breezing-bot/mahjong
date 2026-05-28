use super::{adapter, melds, points, request, tiles, yaku};
use crate::types::{AnalyzeHandRequest, ScoringResult};
use riichi_calc::finder::result::FoundResult;

pub fn analyze_hand(request: AnalyzeHandRequest) -> ScoringResult {
  match analyze_hand_inner(request) {
    Ok(result) => result,
    Err(error) => ScoringResult {
      is_win: false,
      yaku: Vec::new(),
      han: 0,
      fu: 0,
      score_breakdown: None,
      waits: Vec::new(),
      errors: vec![error],
    },
  }
}

fn analyze_hand_inner(request: AnalyzeHandRequest) -> Result<ScoringResult, String> {
  let winning_tile_id = request
    .winning_tile
    .clone()
    .ok_or_else(|| "请先标记和了牌".to_string())?;
  let winning_tile = tiles::parse_tile_id(&winning_tile_id)?;
  let hand_tiles = request::normalize_closed_hand(&request.hand_tiles, request.melds.len())?;
  let hand = hand_tiles
    .iter()
    .map(|tile| tiles::parse_tile_id(tile))
    .collect::<Result<Vec<_>, _>>()?;
  let naki = request
    .melds
    .iter()
    .map(melds::convert_meld)
    .collect::<Result<Vec<_>, _>>()?;
  let dora = request
    .dora_indicators
    .iter()
    .map(|tile| tiles::dora_from_indicator(tile))
    .collect::<Result<Vec<_>, _>>()?;
  let ura_dora = request
    .ura_dora_indicators
    .iter()
    .map(|tile| tiles::dora_from_indicator(tile))
    .collect::<Result<Vec<_>, _>>()?;

  let field = adapter::field(&request, dora);
  let status = adapter::status(&request, ura_dora);
  let output = adapter::input(hand, naki, winning_tile, field, status)
    .calc_hand()
    .map_err(|err| format!("输入不是合法和牌形：{err:?}"))?;

  let is_yakuman = matches!(output.found_result, FoundResult::FoundYakuman(_));
  Ok(ScoringResult {
    is_win: true,
    yaku: yaku::flatten_yaku(&output.found_result),
    han: output.score_result.detail.han,
    fu: output.score_result.detail.fu,
    score_breakdown: Some(points::score_breakdown(
      &output.score_result.actual_points,
      output.score_result.detail.fu,
      output.score_result.detail.han,
      is_yakuman,
    )),
    waits: Vec::new(),
    errors: Vec::new(),
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::types::{MeldInput, MeldKind, RiichiInput, SpecialWinInput, WinMethodInput, WindInput};

  fn base_request() -> AnalyzeHandRequest {
    AnalyzeHandRequest {
      hand_tiles: vec![
        "1m", "2m", "3m", "5m", "6m", "7m", "2p", "3p", "4p", "6s", "7s", "9s", "9s",
      ]
      .into_iter()
      .map(String::from)
      .collect(),
      winning_tile: Some("5s".to_string()),
      melds: Vec::new(),
      self_wind: WindInput::East,
      round_wind: WindInput::East,
      win_method: WinMethodInput::Ron,
      riichi: RiichiInput::Riichi,
      special_win: SpecialWinInput::default(),
      honba: 0,
      dora_indicators: Vec::new(),
      ura_dora_indicators: Vec::new(),
    }
  }

  #[test]
  fn scores_simple_pinfu_riichi_ron() {
    let result = analyze_hand(base_request());

    assert!(result.is_win, "{:?}", result.errors);
    assert!(result.han >= 1);
    assert!(result.fu >= 20);
  }

  #[test]
  fn scores_closed_tsumo_hand() {
    let mut request = base_request();
    request.win_method = WinMethodInput::Tsumo;

    let result = analyze_hand(request);

    assert!(result.is_win, "{:?}", result.errors);
    assert!(result.score_breakdown.and_then(|score| score.tsumo_points).is_some());
  }

  #[test]
  fn rejects_missing_winning_tile() {
    let mut request = base_request();
    request.winning_tile = None;

    let result = analyze_hand(request);

    assert!(!result.is_win);
    assert_eq!(result.errors, vec!["请先标记和了牌"]);
  }

  #[test]
  fn accepts_open_meld_shape() {
    let request = AnalyzeHandRequest {
      hand_tiles: vec!["4m", "5m", "6m", "2p", "3p", "4p", "6s", "7s", "9s", "9s"]
        .into_iter()
        .map(String::from)
        .collect(),
      winning_tile: Some("5s".to_string()),
      melds: vec![MeldInput {
        kind: MeldKind::Pon,
        tiles: vec!["5z", "5z", "5z"].into_iter().map(String::from).collect(),
      }],
      self_wind: WindInput::East,
      round_wind: WindInput::East,
      win_method: WinMethodInput::Ron,
      riichi: RiichiInput::None,
      special_win: SpecialWinInput::default(),
      honba: 0,
      dora_indicators: Vec::new(),
      ura_dora_indicators: Vec::new(),
    };

    let result = analyze_hand(request);

    assert!(result.is_win, "{:?}", result.errors);
  }
}
