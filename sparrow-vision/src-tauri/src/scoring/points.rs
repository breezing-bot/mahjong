use crate::types::{ScoreBreakdown, ScoringResult, TsumoPoints};
use riichi_calc::calculator::result::Points;
use riichi_calc::constants::status::WinMethod;
use riichi_calc::finder::result::FoundResult;

pub fn actual_points(
  fu: u8, han: u8, found: &FoundResult, win_method: &WinMethod, is_dealer: bool, honba: u8,
) -> Points {
  let base = base_points(fu, han, matches!(found, FoundResult::FoundYakuman(_)));
  match (is_dealer, win_method) {
    (true, WinMethod::Ron) => Points::Ron(round_points(base * 6) + honba as u32 * 300),
    (false, WinMethod::Ron) => Points::Ron(round_points(base * 4) + honba as u32 * 300),
    (true, WinMethod::Tumo) => Points::DealerTumo(round_points(base * 2) + honba as u32 * 100),
    (false, WinMethod::Tumo) => Points::ChildTumo(
      round_points(base) + honba as u32 * 100,
      round_points(base * 2) + honba as u32 * 100,
    ),
  }
}

pub fn base_points(fu: u8, han: u8, is_yakuman: bool) -> u32 {
  if is_yakuman {
    return 8000 * han as u32;
  }
  match han {
    0..=4 => (fu as u32 * 2_u32.pow(han as u32 + 2)).min(2000),
    5 => 2000,
    6 | 7 => 3000,
    8..=10 => 4000,
    11 | 12 => 6000,
    _ => 8000,
  }
}

pub fn result_rank(result: &ScoringResult) -> u32 {
  let Some(score) = &result.score_breakdown else {
    return 0;
  };
  if score.ron_points > 0 {
    return score.ron_points;
  }
  score
    .tsumo_points
    .as_ref()
    .map(|tsumo| tsumo.dealer + tsumo.non_dealer * 2)
    .unwrap_or(0)
}

pub fn score_breakdown(points: &Points, fu: u8, han: u8, is_yakuman: bool) -> ScoreBreakdown {
  ScoreBreakdown {
    base_points: base_points(fu, han, is_yakuman),
    ron_points: match points {
      Points::Ron(value) => *value,
      _ => 0,
    },
    tsumo_points: match points {
      Points::ChildTumo(non_dealer, dealer) => Some(TsumoPoints {
        dealer: *dealer,
        non_dealer: *non_dealer,
      }),
      Points::DealerTumo(value) => Some(TsumoPoints {
        dealer: *value,
        non_dealer: *value,
      }),
      Points::Ron(_) => None,
    },
  }
}

fn round_points(points: u32) -> u32 {
  points.div_ceil(100) * 100
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rounds_points_up_to_hundred() {
    assert_eq!(round_points(960), 1000);
    assert_eq!(round_points(1000), 1000);
  }

  #[test]
  fn calculates_limit_base_points() {
    assert_eq!(base_points(30, 4, false), 1920);
    assert_eq!(base_points(40, 4, false), 2000);
    assert_eq!(base_points(30, 5, false), 2000);
    assert_eq!(base_points(30, 13, false), 8000);
    assert_eq!(base_points(30, 1, true), 8000);
  }
}
