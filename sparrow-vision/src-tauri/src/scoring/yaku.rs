use crate::types::YakuResult;
use riichi_calc::finder::result::{FoundResult, FoundYaku, FoundYakuman};
use riichi_calc::finder::yaku::YakuEntry;

pub fn flatten_yaku(found: &FoundResult) -> Vec<YakuResult> {
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
