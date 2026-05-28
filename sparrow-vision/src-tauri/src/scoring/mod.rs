mod calculator;
mod melds;
mod tiles;
mod yaku;

use crate::types::{AnalyzeHandRequest, ScoringResult};

pub fn analyze_hand(request: AnalyzeHandRequest) -> ScoringResult {
  calculator::analyze_hand(request)
}
