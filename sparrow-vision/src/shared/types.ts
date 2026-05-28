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
  hand_tiles: TileId[];
  winning_tile: TileId | null;
  melds: MeldInput[];
  self_wind: WindInput;
  round_wind: WindInput;
  win_method: WinMethodInput;
  riichi: RiichiInput;
  special_win: SpecialWinInput;
  honba: number;
  dora_indicators: TileId[];
  ura_dora_indicators: TileId[];
}

export interface ScoringResult {
  is_win: boolean;
  yaku: Array<{ id: string; name: string; han: number }>;
  han: number;
  fu: number;
  score_breakdown: {
    base_points: number;
    ron_points: number;
    tsumo_points: { dealer: number; non_dealer: number } | null;
  } | null;
  waits: string[];
  errors: string[];
}
