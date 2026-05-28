use crate::types::{ScoreBreakdown, TsumoPoints};
use riichi_calc::calculator::result::Points;

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn calculates_limit_base_points() {
    assert_eq!(base_points(30, 4, false), 1920);
    assert_eq!(base_points(40, 4, false), 2000);
    assert_eq!(base_points(30, 5, false), 2000);
    assert_eq!(base_points(30, 13, false), 8000);
    assert_eq!(base_points(30, 1, true), 8000);
  }
}
