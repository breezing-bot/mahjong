use crate::types::YakuSummary;
use riichi_calc::finder::result::{FoundResult, FoundYaku, FoundYakuman};
use riichi_calc::finder::yaku::YakuEntry;

pub fn flatten_yaku(found: &FoundResult) -> Vec<YakuSummary> {
  let mut yaku = Vec::new();
  match found {
    FoundResult::FoundYaku(FoundYaku {
      dora,
      ii_han,
      ryan_han,
      san_han,
      roku_han,
    }) => {
      push_yaku(&mut yaku, dora);
      push_yaku(&mut yaku, ii_han);
      push_yaku(&mut yaku, ryan_han);
      push_yaku(&mut yaku, san_han);
      push_yaku(&mut yaku, roku_han);
    }
    FoundResult::FoundYakuman(FoundYakuman { yakuman }) => {
      push_yaku(&mut yaku, yakuman);
    }
  }
  yaku
}

fn push_yaku(target: &mut Vec<YakuSummary>, source: &[YakuEntry]) {
  for entry in source {
    target.push(YakuSummary {
      name: entry.name().to_string(),
      han: entry.value,
    });
  }
}
