export type TileId =
  | "m5r"
  | "m1"
  | "m2"
  | "m3"
  | "m4"
  | "m5"
  | "m6"
  | "m7"
  | "m8"
  | "m9"
  | "p5r"
  | "p1"
  | "p2"
  | "p3"
  | "p4"
  | "p5"
  | "p6"
  | "p7"
  | "p8"
  | "p9"
  | "s5r"
  | "s1"
  | "s2"
  | "s3"
  | "s4"
  | "s5"
  | "s6"
  | "s7"
  | "s8"
  | "s9"
  | "z1"
  | "z2"
  | "z3"
  | "z4"
  | "z5"
  | "z6"
  | "z7"
  | "f1"
  | "f2"
  | "f3"
  | "f4"
  | "f5"
  | "f6"
  | "f7"
  | "f8";

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
    dropped_duplicate_count: number;
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
