mod adapter;
mod melds;
mod points;
mod request;
mod tiles;

use crate::types::{AnalyzeHandRequest, ScoreBreakdown, ScoringResult, TsumoPoints, WindInput, YakuResult};
use riichi_calc::calculator::score::calc_score;
use riichi_calc::finder::finder::Finder;
use riichi_calc::finder::result::{FoundResult, FoundYaku, FoundYakuman};
use riichi_calc::finder::yaku::YakuEntry;

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
  let hand_tiles = request::normalize_closed_hand(&request.hand_tiles, &winning_tile_id, request.melds.len())?;
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
  let input = adapter::input(hand, naki, winning_tile, field.clone(), status.clone());

  let parsed = input
    .parse_hand()
    .map_err(|err| format!("输入不是合法和牌形：{err:?}"))?;

  let mut best: Option<ScoringResult> = None;
  for hand in parsed {
    let found = Finder::find_hand(&hand);
    if !found.is_valid_hora() {
      continue;
    }

    let score = calc_score(&found, &field, &hand.winning_hand, &status);
    let actual_points = points::actual_points(
      score.detail.fu,
      score.detail.han,
      &found,
      &status.win_method,
      request.self_wind == WindInput::East,
      request.honba,
    );
    let is_yakuman = matches!(found, FoundResult::FoundYakuman(_));
    let candidate = ScoringResult {
      is_win: true,
      yaku: flatten_yaku(&found),
      han: score.detail.han,
      fu: score.detail.fu,
      score_breakdown: Some(score_breakdown(
        &actual_points,
        score.detail.fu,
        score.detail.han,
        is_yakuman,
      )),
      waits: Vec::new(),
      errors: Vec::new(),
    };

    if best
      .as_ref()
      .map(|current| points::result_rank(current) < points::result_rank(&candidate))
      .unwrap_or(true)
    {
      best = Some(candidate);
    }
  }

  best.ok_or_else(|| "没有找到役种，无法和牌".to_string())
}

fn flatten_yaku(found: &FoundResult) -> Vec<YakuResult> {
  let mut yaku = Vec::new();
  match found {
    FoundResult::FoundYaku(FoundYaku {
      dora,
      ii_han,
      ryan_han,
      san_han,
      roku_han,
    }) => {
      push_yaku(&mut yaku, dora, "dora");
      push_yaku(&mut yaku, ii_han, "one_han");
      push_yaku(&mut yaku, ryan_han, "two_han");
      push_yaku(&mut yaku, san_han, "three_han");
      push_yaku(&mut yaku, roku_han, "six_han");
    }
    FoundResult::FoundYakuman(FoundYakuman { yakuman }) => {
      push_yaku(&mut yaku, yakuman, "yakuman");
    }
  }
  yaku
}

fn push_yaku(target: &mut Vec<YakuResult>, source: &[YakuEntry], prefix: &str) {
  for entry in source {
    target.push(YakuResult {
      id: format!("{prefix}_{}", target.len() + 1),
      name: entry.name().to_string(),
      han: entry.value,
    });
  }
}

fn score_breakdown(
  points: &riichi_calc::calculator::result::Points, fu: u8, han: u8, is_yakuman: bool,
) -> ScoreBreakdown {
  ScoreBreakdown {
    base_points: points::base_points(fu, han, is_yakuman),
    ron_points: match points {
      riichi_calc::calculator::result::Points::Ron(value) => *value,
      _ => 0,
    },
    tsumo_points: match points {
      riichi_calc::calculator::result::Points::ChildTumo(non_dealer, dealer) => Some(TsumoPoints {
        dealer: *dealer,
        non_dealer: *non_dealer,
      }),
      riichi_calc::calculator::result::Points::DealerTumo(value) => Some(TsumoPoints {
        dealer: *value,
        non_dealer: *value,
      }),
      riichi_calc::calculator::result::Points::Ron(_) => None,
    },
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::types::{MeldInput, MeldKind, RiichiInput, SpecialWinInput, WinMethodInput, WindInput};

  fn base_request() -> AnalyzeHandRequest {
    AnalyzeHandRequest {
      hand_tiles: vec![
        "m1", "m2", "m3", "m5", "m6", "m7", "p2", "p3", "p4", "s6", "s7", "s9", "s9",
      ]
      .into_iter()
      .map(String::from)
      .collect(),
      winning_tile: Some("s5".to_string()),
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
      hand_tiles: vec!["m4", "m5", "m6", "p2", "p3", "p4", "s6", "s7", "s9", "s9"]
        .into_iter()
        .map(String::from)
        .collect(),
      winning_tile: Some("s5".to_string()),
      melds: vec![MeldInput {
        kind: MeldKind::Pon,
        tiles: vec!["z5", "z5", "z5"].into_iter().map(String::from).collect(),
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
