import type { TileId } from "./types";
import { tileLabel, tileTone } from "./tiles";

interface TileBadgeProps {
  tile: TileId;
  selected?: boolean;
  muted?: boolean;
  onClick?: () => void;
}

export function TileBadge({ tile, selected, muted, onClick }: TileBadgeProps) {
  return (
    <button
      className={`tile tile-${tileTone(tile)}${selected ? " tile-selected" : ""}${
        muted ? " tile-muted" : ""
      }`}
      type="button"
      onClick={onClick}
      title={tile}
    >
      {tileLabel(tile)}
    </button>
  );
}
