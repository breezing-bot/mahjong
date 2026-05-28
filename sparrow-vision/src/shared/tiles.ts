import type { TileId } from "./types";

export const TILE_IDS = [
  "0m",
  "1m",
  "2m",
  "3m",
  "4m",
  "5m",
  "6m",
  "7m",
  "8m",
  "9m",
  "0p",
  "1p",
  "2p",
  "3p",
  "4p",
  "5p",
  "6p",
  "7p",
  "8p",
  "9p",
  "0s",
  "1s",
  "2s",
  "3s",
  "4s",
  "5s",
  "6s",
  "7s",
  "8s",
  "9s",
  "1z",
  "2z",
  "3z",
  "4z",
  "5z",
  "6z",
  "7z",
  "1f",
  "2f",
  "3f",
  "4f",
  "5f",
  "6f",
  "7f",
  "8f",
] as const satisfies readonly TileId[];

export const SCORING_TILE_IDS = TILE_IDS.filter((tile) => !tile.endsWith("f"));

export const WIND_OPTIONS = [
  ["east", "东"],
  ["south", "南"],
  ["west", "西"],
  ["north", "北"],
] as const;

const HONOR_LABELS: Record<string, string> = {
  "1z": "东",
  "2z": "南",
  "3z": "西",
  "4z": "北",
  "5z": "白",
  "6z": "发",
  "7z": "中",
};

export function tileLabel(tile: TileId): string {
  if (tile[0] === "0") return `赤5${suitLabel(tile[1])}`;
  if (tile.endsWith("z")) return HONOR_LABELS[tile] ?? tile;
  if (tile.endsWith("f")) return `花${tile[0]}`;
  return `${tile[0]}${suitLabel(tile[1])}`;
}

export function tileTone(tile: TileId): string {
  if (tile.endsWith("m")) return "manzu";
  if (tile.endsWith("p")) return "pinzu";
  if (tile.endsWith("s")) return "souzu";
  if (tile.endsWith("z")) return "honor";
  return "flower";
}

function suitLabel(suit: string): string {
  return suit === "m" ? "万" : suit === "p" ? "筒" : "索";
}
