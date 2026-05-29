mod calculator;
mod melds;
mod yaku;

use crate::types::{AnalyzeHandRequest, AnalyzeHandResult};

pub fn analyze_hand(request: AnalyzeHandRequest) -> Result<AnalyzeHandResult, String> {
  calculator::analyze_hand(request)
}
