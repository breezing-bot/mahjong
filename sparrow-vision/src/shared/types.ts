export type TileId =
  | "0m"
  | "1m"
  | "2m"
  | "3m"
  | "4m"
  | "5m"
  | "6m"
  | "7m"
  | "8m"
  | "9m"
  | "0p"
  | "1p"
  | "2p"
  | "3p"
  | "4p"
  | "5p"
  | "6p"
  | "7p"
  | "8p"
  | "9p"
  | "0s"
  | "1s"
  | "2s"
  | "3s"
  | "4s"
  | "5s"
  | "6s"
  | "7s"
  | "8s"
  | "9s"
  | "1z"
  | "2z"
  | "3z"
  | "4z"
  | "5z"
  | "6z"
  | "7z"
  | "1f"
  | "2f"
  | "3f"
  | "4f"
  | "5f"
  | "6f"
  | "7f"
  | "8f";

export interface BBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Detection {
  tile_id: TileId;
  confidence: number;
  bbox: BBox;
}

export interface RecognitionResult {
  detections: Detection[];
  hand_tiles: TileId[];
  quality_flags: string[];
  diagnostics: {
    raw_detection_count: number;
    kept_detection_count: number;
    dropped_low_confidence_count: number;
    dropped_unknown_class_count: number;
  };
}

export type MeldKind = "chi" | "pon" | "daiminkan" | "ankan";
export type WindInput = "east" | "south" | "west" | "north";
export type WinMethodInput = "ron" | "tsumo";
export type RiichiInput = "none" | "riichi" | "double_riichi";

export interface MeldInput {
  kind: MeldKind;
  tiles: TileId[];
}

export interface SpecialWinInput {
  ippatsu: boolean;
  chankan: boolean;
  rinshan: boolean;
  haitei: boolean;
  hotei: boolean;
  first_turn_tsumo: boolean;
}

export interface AnalyzeHandRequest {
  /** 闭手牌，不包含和了牌；对应 PiInput.hand。 */
  hand: TileId[];
  /** 和了牌，对应 riichi-calc 的 hora。 */
  hora: TileId;
  /** 副露面子，包括吃、碰、明杠、暗杠；对应 PiInput.naki。 */
  naki: MeldInput[];
  /** 自风，也就是玩家座风；对应 Field.zikaze。 */
  zikaze: WindInput;
  /** 场风；对应 Field.bakaze。 */
  bakaze: WindInput;
  /** 和牌方式：荣和或自摸。 */
  win_method: WinMethodInput;
  /** 立直状态：无立直、立直或双立直。 */
  riichi: RiichiInput;
  /** 一发、抢杠、岭上、海底等特殊役状态；对应 Status.special_win。 */
  special_win: SpecialWinInput;
  /** 本场数；对应 Field.honba。 */
  honba: number;
  /** 宝牌指示牌，后端会转换为实际宝牌；对应 Field.dora。 */
  dora: TileId[];
  /** 里宝牌指示牌，仅在立直相关状态下参与计算；对应 RiichiStatus 中的 ura dora。 */
  ura_dora: TileId[];
}

export interface ScoringResult {
  is_win: boolean;
  yaku: Array<{ id: string; name: string; han: number }>;
  han: number;
  fu: number;
  ron_points: number | null;
  tsumo_points: { dealer: number; non_dealer: number } | null;
  waits: string[];
  errors: string[];
}
