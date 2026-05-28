import type { MeldInput, MeldKind, TileId } from "../../shared/types";
import { TileBadge } from "../../shared/TileBadge";
import { SCORING_TILE_IDS, tileLabel } from "../../shared/tiles";

interface HandEditorProps {
  handTiles: TileId[];
  winningTile: TileId | null;
  melds: MeldInput[];
  pendingTiles: TileId[];
  onHandTilesChange: (tiles: TileId[]) => void;
  onWinningTileChange: (tile: TileId | null) => void;
  onMeldsChange: (melds: MeldInput[]) => void;
  onPendingTilesChange: (tiles: TileId[]) => void;
}

export function HandEditor({
  handTiles,
  winningTile,
  melds,
  pendingTiles,
  onHandTilesChange,
  onWinningTileChange,
  onMeldsChange,
  onPendingTilesChange,
}: HandEditorProps) {
  return (
    <section className="panel editor-panel">
      <div className="panel-heading">
        <h2>牌面编辑</h2>
        <span className="meta">{handTiles.length} 张手牌</span>
      </div>
      <TileRack
        title="手牌"
        tiles={handTiles}
        winningTile={winningTile}
        onTilesChange={onHandTilesChange}
        onWinningTileChange={onWinningTileChange}
      />
      <MeldEditor melds={melds} onChange={onMeldsChange} />
      <TileRack
        title="待确认"
        tiles={pendingTiles}
        winningTile={winningTile}
        onTilesChange={onPendingTilesChange}
        onWinningTileChange={onWinningTileChange}
      />
    </section>
  );
}

function TileRack({
  title,
  tiles,
  winningTile,
  onTilesChange,
  onWinningTileChange,
}: {
  title: string;
  tiles: TileId[];
  winningTile: TileId | null;
  onTilesChange: (tiles: TileId[]) => void;
  onWinningTileChange: (tile: TileId | null) => void;
}) {
  return (
    <div className="rack">
      <div className="rack-heading">
        <h3>{title}</h3>
        <TileSelect
          label="添加"
          onPick={(tile) => onTilesChange([...tiles, tile])}
        />
      </div>
      <div className="tile-row">
        {tiles.map((tile, index) => (
          <div className="tile-edit" key={`${title}-${tile}-${index}`}>
            <TileBadge
              selected={winningTile === tile}
              tile={tile}
              onClick={() =>
                onWinningTileChange(winningTile === tile ? null : tile)
              }
            />
            <select
              aria-label="修改牌"
              value={tile}
              onChange={(event) => {
                const next = [...tiles];
                next[index] = event.currentTarget.value as TileId;
                onTilesChange(next);
              }}
            >
              {SCORING_TILE_IDS.map((option) => (
                <option key={option} value={option}>
                  {tileLabel(option)}
                </option>
              ))}
            </select>
            <div className="mini-actions">
              <button
                type="button"
                title="左移"
                onClick={() => onTilesChange(move(tiles, index, -1))}
              >
                ‹
              </button>
              <button
                type="button"
                title="右移"
                onClick={() => onTilesChange(move(tiles, index, 1))}
              >
                ›
              </button>
              <button
                type="button"
                title="删除"
                onClick={() => onTilesChange(tiles.filter((_, i) => i !== index))}
              >
                ×
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function MeldEditor({
  melds,
  onChange,
}: {
  melds: MeldInput[];
  onChange: (melds: MeldInput[]) => void;
}) {
  return (
    <div className="rack">
      <div className="rack-heading">
        <h3>副露</h3>
        <button
          type="button"
          onClick={() =>
            onChange([...melds, { kind: "pon", tiles: ["5z", "5z", "5z"] }])
          }
        >
          添加副露
        </button>
      </div>
      <div className="meld-list">
        {melds.map((meld, meldIndex) => (
          <div className="meld-row" key={`meld-${meldIndex}`}>
            <select
              aria-label="副露类型"
              value={meld.kind}
              onChange={(event) => {
                const next = [...melds];
                next[meldIndex] = {
                  ...meld,
                  kind: event.currentTarget.value as MeldKind,
                };
                onChange(next);
              }}
            >
              <option value="chi">吃</option>
              <option value="pon">碰</option>
              <option value="daiminkan">明杠</option>
              <option value="ankan">暗杠</option>
            </select>
            {meld.tiles.map((tile, tileIndex) => (
              <select
                aria-label="副露牌"
                key={`${meldIndex}-${tileIndex}`}
                value={tile}
                onChange={(event) => {
                  const next = [...melds];
                  const tiles = [...meld.tiles];
                  tiles[tileIndex] = event.currentTarget.value as TileId;
                  next[meldIndex] = { ...meld, tiles };
                  onChange(next);
                }}
              >
                {SCORING_TILE_IDS.map((option) => (
                  <option key={option} value={option}>
                    {tileLabel(option)}
                  </option>
                ))}
              </select>
            ))}
            <TileSelect
              label="+"
              onPick={(tile) => {
                const next = [...melds];
                next[meldIndex] = { ...meld, tiles: [...meld.tiles, tile] };
                onChange(next);
              }}
            />
            <button
              type="button"
              onClick={() => onChange(melds.filter((_, i) => i !== meldIndex))}
            >
              删除
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}

function TileSelect({
  label,
  onPick,
}: {
  label: string;
  onPick: (tile: TileId) => void;
}) {
  return (
    <select
      aria-label={label}
      defaultValue=""
      onChange={(event) => {
        const tile = event.currentTarget.value as TileId;
        if (tile) {
          onPick(tile);
          event.currentTarget.value = "";
        }
      }}
    >
      <option value="" disabled>
        {label}
      </option>
      {SCORING_TILE_IDS.map((tile) => (
        <option key={tile} value={tile}>
          {tileLabel(tile)}
        </option>
      ))}
    </select>
  );
}

function move<T>(items: T[], index: number, direction: -1 | 1): T[] {
  const nextIndex = index + direction;
  if (nextIndex < 0 || nextIndex >= items.length) {
    return items;
  }
  const next = [...items];
  [next[index], next[nextIndex]] = [next[nextIndex], next[index]];
  return next;
}
