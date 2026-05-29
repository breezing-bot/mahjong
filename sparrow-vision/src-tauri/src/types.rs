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
  pub id: u32,
  pub tile_id: TileId,
  pub confidence: f32,
  pub bbox: BBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecognitionMeldKind {
  Chi,
  Pon,
  Daiminkan,
  Ankan,
  Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognitionMeld {
  pub id: String,
  pub kind: RecognitionMeldKind,
  pub tiles: Vec<u32>,
  pub needs_confirmation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognitionLayout {
  pub hand: Vec<u32>,
  pub hora: Option<u32>,
  pub naki: Vec<RecognitionMeld>,
  pub unassigned: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognitionResult {
  pub detections: Vec<Detection>,
  pub layout: RecognitionLayout,
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
  /// 闭手牌，不包含和了牌；对应 PiInput.hand。
  pub hand: Vec<TileId>,
  /// 和了牌，对应 riichi-calc 的 hora。
  pub hora: TileId,
  /// 副露面子，包括吃、碰、明杠、暗杠；对应 PiInput.naki。
  pub naki: Vec<MeldInput>,
  /// 自风，也就是玩家座风；对应 Field.zikaze。
  pub zikaze: WindInput,
  /// 场风；对应 Field.bakaze。
  pub bakaze: WindInput,
  /// 和牌方式：荣和或自摸。
  pub win_method: WinMethodInput,
  /// 立直状态：无立直、立直或双立直。
  pub riichi: RiichiInput,
  /// 一发、抢杠、岭上、海底等特殊役状态；对应 Status.special_win。
  pub special_win: SpecialWinInput,
  /// 本场数；对应 Field.honba。
  pub honba: u8,
  /// 宝牌指示牌，后端会转换为实际宝牌；对应 Field.dora。
  pub dora: Vec<TileId>,
  /// 里宝牌指示牌，仅在立直相关状态下参与计算；对应 RiichiStatus 中的 ura dora。
  pub ura_dora: Vec<TileId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YakuSummary {
  pub name: String,
  pub han: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ScorePoints {
  Ron { points: u32 },
  Tsumo { dealer: u32, non_dealer: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeHandResult {
  pub yaku: Vec<YakuSummary>,
  pub han: u8,
  pub fu: u8,
  pub points: ScorePoints,
}
