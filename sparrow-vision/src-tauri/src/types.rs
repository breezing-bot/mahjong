use serde::{Deserialize, Serialize};

pub type TileId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BBox {
  pub x: f32,
  pub y: f32,
  pub width: f32,
  pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
  pub tile_id: TileId,
  pub confidence: f32,
  pub bbox: BBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognitionDiagnostics {
  pub raw_detection_count: usize,
  pub kept_detection_count: usize,
  pub dropped_low_confidence_count: usize,
  pub dropped_unknown_class_count: usize,
  pub dropped_duplicate_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognitionResult {
  pub detections: Vec<Detection>,
  pub hand_tiles: Vec<TileId>,
  pub quality_flags: Vec<String>,
  pub diagnostics: RecognitionDiagnostics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeldKind {
  Chi,
  Pon,
  Daiminkan,
  Ankan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeldInput {
  pub kind: MeldKind,
  pub tiles: Vec<TileId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WindInput {
  East,
  South,
  West,
  North,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WinMethodInput {
  Ron,
  Tsumo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiichiInput {
  None,
  Riichi,
  DoubleRiichi,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpecialWinInput {
  pub ippatsu: bool,
  pub chankan: bool,
  pub rinshan: bool,
  pub haitei: bool,
  pub hotei: bool,
  pub first_turn_tsumo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeHandRequest {
  pub hand_tiles: Vec<TileId>,
  pub winning_tile: Option<TileId>,
  pub melds: Vec<MeldInput>,
  pub self_wind: WindInput,
  pub round_wind: WindInput,
  pub win_method: WinMethodInput,
  pub riichi: RiichiInput,
  pub special_win: SpecialWinInput,
  pub honba: u8,
  pub dora_indicators: Vec<TileId>,
  pub ura_dora_indicators: Vec<TileId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YakuResult {
  pub id: String,
  pub name: String,
  pub han: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsumoPoints {
  pub dealer: u32,
  pub non_dealer: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
  pub base_points: u32,
  pub ron_points: u32,
  pub tsumo_points: Option<TsumoPoints>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringResult {
  pub is_win: bool,
  pub yaku: Vec<YakuResult>,
  pub han: u8,
  pub fu: u8,
  pub score_breakdown: Option<ScoreBreakdown>,
  pub waits: Vec<String>,
  pub errors: Vec<String>,
}
