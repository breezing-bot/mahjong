import type { TileId } from "./types";

export const TILE_IDS = [
  "m5r",
  "m1",
  "m2",
  "m3",
  "m4",
  "m5",
  "m6",
  "m7",
  "m8",
  "m9",
  "p5r",
  "p1",
  "p2",
  "p3",
  "p4",
  "p5",
  "p6",
  "p7",
  "p8",
  "p9",
  "s5r",
  "s1",
  "s2",
  "s3",
  "s4",
  "s5",
  "s6",
  "s7",
  "s8",
  "s9",
  "z1",
  "z2",
  "z3",
  "z4",
  "z5",
  "z6",
  "z7",
  "f1",
  "f2",
  "f3",
  "f4",
  "f5",
  "f6",
  "f7",
  "f8",
] as const satisfies readonly TileId[];

export const WIND_OPTIONS = [
  ["east", "东"],
  ["south", "南"],
  ["west", "西"],
  ["north", "北"],
] as const;

export function tileLabel(tile: TileId): string {
  if (tile.endsWith("r")) {
    return `${tile[1]}赤${suitLabel(tile[0])}`;
  }
  if (tile.startsWith("z")) {
    return (
      {
        z1: "东",
        z2: "南",
        z3: "西",
        z4: "北",
        z5: "白",
        z6: "发",
        z7: "中",
      } as Record<string, string>
    )[tile];
  }
  if (tile.startsWith("f")) {
    return `花${tile.slice(1)}`;
  }
  return `${tile[1]}${suitLabel(tile[0])}`;
}

export function tileTone(tile: TileId): string {
  if (tile.startsWith("m")) return "manzu";
  if (tile.startsWith("p")) return "pinzu";
  if (tile.startsWith("s")) return "souzu";
  if (tile.startsWith("z")) return "honor";
  return "flower";
}

function suitLabel(suit: string): string {
  return suit === "m" ? "万" : suit === "p" ? "筒" : "索";
}
