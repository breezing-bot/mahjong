import type {
  RecognitionMeld,
  RecognitionMeldKind,
  RecognitionResult,
  TileId,
} from "../../shared/types";
import { TileBadge } from "../../shared/TileBadge";
import { SCORING_TILE_IDS, tileLabel } from "../../shared/tiles";

interface HandEditorProps {
  recognition: RecognitionResult | null;
  selectedTileId: number | null;
  onRecognitionChange: (recognition: RecognitionResult) => void;
  onSelectedTileChange: (tileId: number | null) => void;
}

export function HandEditor({
  recognition,
  selectedTileId,
  onRecognitionChange,
  onSelectedTileChange,
}: HandEditorProps) {
  if (!recognition) {
    return (
      <section className="panel editor-panel">
        <div className="panel-heading">
          <h2>牌面编辑</h2>
          <span className="meta">暂无牌</span>
        </div>
        <div className="result-empty">上传照片后可编辑识别出的牌。</div>
      </section>
    );
  }

  const tiles = tileMap(recognition);
  const selectedTile = selectedTileId === null ? undefined : tiles.get(selectedTileId);

  function update(next: RecognitionResult) {
    onRecognitionChange(next);
  }

  function assignSelected(target: AssignmentTarget) {
    if (selectedTileId === null) return;
    update(assignTile(recognition!, selectedTileId, target));
  }

  return (
    <section className="panel editor-panel">
      <div className="panel-heading">
        <h2>牌面编辑</h2>
        <span className="meta">{recognition.detections.length} 张识别牌</span>
      </div>

      <div className="assignment-bar">
        <div>
          <strong>{selectedTile ? tileLabel(selectedTile.tile_id) : "未选择牌"}</strong>
          {selectedTile ? <span className="muted-text"> {selectedTile.id}</span> : null}
        </div>
        <div className="assignment-actions">
          <button type="button" disabled={selectedTileId === null} onClick={() => assignSelected({ kind: "closed" })}>
            闭手牌
          </button>
          <button type="button" disabled={selectedTileId === null} onClick={() => assignSelected({ kind: "winning" })}>
            和了牌
          </button>
          <button type="button" disabled={selectedTileId === null} onClick={() => assignSelected({ kind: "unassigned" })}>
            待确认
          </button>
          <button type="button" disabled={selectedTileId === null} onClick={() => assignSelected({ kind: "new-meld" })}>
            新副露
          </button>
        </div>
      </div>

      <TileRack
        title="闭手牌"
        ids={recognition.layout.hand}
        recognition={recognition}
        selectedTileId={selectedTileId}
        onSelect={onSelectedTileChange}
        onChange={(ids) =>
          update({ ...recognition, layout: { ...recognition.layout, hand: ids } })
        }
        onTileChange={(tileId, tile) => updateTile(recognition, tileId, tile, update)}
        onRemove={(tileId) => update(assignTile(recognition, tileId, { kind: "unassigned" }))}
      />

      <div className="rack">
        <div className="rack-heading">
          <h3>和了牌</h3>
          <button
            type="button"
            disabled={recognition.layout.hora === null}
            onClick={() => {
              const tileId = recognition.layout.hora;
              if (tileId !== null) update(assignTile(recognition, tileId, { kind: "unassigned" }));
            }}
          >
            清空
          </button>
        </div>
        <div className="tile-row">
          {recognition.layout.hora !== null ? (
            <EditableTile
              id={recognition.layout.hora}
              recognition={recognition}
              selected={selectedTileId === recognition.layout.hora}
              onSelect={onSelectedTileChange}
              onTileChange={(tileId, tile) => updateTile(recognition, tileId, tile, update)}
              onRemove={(tileId) => update(assignTile(recognition, tileId, { kind: "unassigned" }))}
            />
          ) : (
            <span className="result-empty">请选择一张和了牌。</span>
          )}
        </div>
      </div>

      <MeldEditor
        recognition={recognition}
        selectedTileId={selectedTileId}
        onSelect={onSelectedTileChange}
        onChange={update}
      />

      <TileRack
        title="待确认"
        ids={recognition.layout.unassigned}
        recognition={recognition}
        selectedTileId={selectedTileId}
        onSelect={onSelectedTileChange}
        onChange={(ids) =>
          update({ ...recognition, layout: { ...recognition.layout, unassigned: ids } })
        }
        onTileChange={(tileId, tile) => updateTile(recognition, tileId, tile, update)}
        onRemove={(tileId) => update(deleteTile(recognition, tileId))}
      />
    </section>
  );
}

function TileRack({
  title,
  ids,
  recognition,
  selectedTileId,
  onSelect,
  onChange,
  onTileChange,
  onRemove,
}: {
  title: string;
  ids: number[];
  recognition: RecognitionResult;
  selectedTileId: number | null;
  onSelect: (id: number | null) => void;
  onChange: (ids: number[]) => void;
  onTileChange: (id: number, tile: TileId) => void;
  onRemove: (id: number) => void;
}) {
  return (
    <div className="rack">
      <div className="rack-heading">
        <h3>{title}</h3>
        <span className="meta">{ids.length}</span>
      </div>
      <div className="tile-row">
        {ids.map((id, index) => (
          <EditableTile
            id={id}
            key={id}
            recognition={recognition}
            selected={selectedTileId === id}
            onSelect={onSelect}
            onTileChange={onTileChange}
            onRemove={onRemove}
            onMove={(direction) => onChange(move(ids, index, direction))}
          />
        ))}
      </div>
    </div>
  );
}

function EditableTile({
  id,
  recognition,
  selected,
  onSelect,
  onTileChange,
  onRemove,
  onMove,
}: {
  id: number;
  recognition: RecognitionResult;
  selected: boolean;
  onSelect: (id: number | null) => void;
  onTileChange: (id: number, tile: TileId) => void;
  onRemove: (id: number) => void;
  onMove?: (direction: -1 | 1) => void;
}) {
  const tile = recognition.detections.find((candidate) => candidate.id === id);
  if (!tile) return null;

  return (
    <div className="tile-edit">
      <TileBadge selected={selected} tile={tile.tile_id} onClick={() => onSelect(id)} />
      <select
        aria-label="修改牌"
        value={tile.tile_id}
        onChange={(event) => onTileChange(id, event.currentTarget.value as TileId)}
      >
        {SCORING_TILE_IDS.map((option) => (
          <option key={option} value={option}>
            {tileLabel(option)}
          </option>
        ))}
      </select>
      <div className="mini-actions">
        <button type="button" title="左移" disabled={!onMove} onClick={() => onMove?.(-1)}>
          &lt;
        </button>
        <button type="button" title="右移" disabled={!onMove} onClick={() => onMove?.(1)}>
          &gt;
        </button>
        <button type="button" title="移除" onClick={() => onRemove(id)}>
          x
        </button>
      </div>
    </div>
  );
}

function MeldEditor({
  recognition,
  selectedTileId,
  onSelect,
  onChange,
}: {
  recognition: RecognitionResult;
  selectedTileId: number | null;
  onSelect: (id: number | null) => void;
  onChange: (recognition: RecognitionResult) => void;
}) {
  return (
    <div className="rack">
      <div className="rack-heading">
        <h3>副露</h3>
        <button
          type="button"
          disabled={selectedTileId === null}
          onClick={() =>
            selectedTileId !== null && onChange(assignTile(recognition, selectedTileId, { kind: "new-meld" }))
          }
        >
          新副露
        </button>
      </div>
      <div className="meld-list">
        {recognition.layout.naki.map((meld) => (
          <MeldRow
            key={meld.id}
            meld={meld}
            recognition={recognition}
            selectedTileId={selectedTileId}
            onSelect={onSelect}
            onChange={onChange}
          />
        ))}
      </div>
    </div>
  );
}

function MeldRow({
  meld,
  recognition,
  selectedTileId,
  onSelect,
  onChange,
}: {
  meld: RecognitionMeld;
  recognition: RecognitionResult;
  selectedTileId: number | null;
  onSelect: (id: number | null) => void;
  onChange: (recognition: RecognitionResult) => void;
}) {
  function updateMeld(nextMeld: RecognitionMeld) {
    onChange({
      ...recognition,
      layout: {
        ...recognition.layout,
        naki: recognition.layout.naki.map((candidate) =>
          candidate.id === meld.id ? nextMeld : candidate,
        ),
      },
    });
  }

  return (
    <div className={`meld-row${meld.needs_confirmation ? " needs-confirmation" : ""}`}>
      <select
        aria-label="副露类型"
        value={meld.kind}
        onChange={(event) =>
          updateMeld({
            ...meld,
            kind: event.currentTarget.value as RecognitionMeldKind,
            needs_confirmation: event.currentTarget.value === "unknown",
          })
        }
      >
        <option value="unknown">待确认</option>
        <option value="chi">吃</option>
        <option value="pon">碰</option>
        <option value="daiminkan">明杠</option>
        <option value="ankan">暗杠</option>
      </select>
      {meld.tiles.map((id, index) => (
        <EditableTile
          id={id}
          key={id}
          recognition={recognition}
          selected={selectedTileId === id}
          onSelect={onSelect}
          onTileChange={(tileId, tile) => updateTile(recognition, tileId, tile, onChange)}
          onRemove={(tileId) => onChange(assignTile(recognition, tileId, { kind: "unassigned" }))}
          onMove={(direction) => updateMeld({ ...meld, tiles: move(meld.tiles, index, direction) })}
        />
      ))}
      <button
        type="button"
        disabled={selectedTileId === null}
        onClick={() =>
          selectedTileId !== null &&
          onChange(assignTile(recognition, selectedTileId, { kind: "meld", meldId: meld.id }))
        }
      >
        加入选中牌
      </button>
      <button
        type="button"
        onClick={() =>
          onChange({
            ...recognition,
            layout: {
              ...recognition.layout,
              naki: recognition.layout.naki.filter((candidate) => candidate.id !== meld.id),
              unassigned: [...recognition.layout.unassigned, ...meld.tiles],
            },
          })
        }
      >
        删除
      </button>
    </div>
  );
}

type AssignmentTarget =
  | { kind: "closed" }
  | { kind: "winning" }
  | { kind: "unassigned" }
  | { kind: "new-meld" }
  | { kind: "meld"; meldId: string };

function assignTile(
  recognition: RecognitionResult,
  tileId: number,
  target: AssignmentTarget,
): RecognitionResult {
  const layout = recognition.layout;
  const nextNaki = layout.naki.map((meld) => ({
    ...meld,
    tiles: meld.tiles.filter((id) => id !== tileId),
  }));
  const nextLayout = {
    ...layout,
    hand: layout.hand.filter((id) => id !== tileId),
    hora: layout.hora === tileId ? null : layout.hora,
    naki: nextNaki,
    unassigned: layout.unassigned.filter((id) => id !== tileId),
  };

  if (target.kind === "closed") {
    nextLayout.hand = [...nextLayout.hand, tileId];
  } else if (target.kind === "winning") {
    if (nextLayout.hora !== null) {
      nextLayout.unassigned = [...nextLayout.unassigned, nextLayout.hora];
    }
    nextLayout.hora = tileId;
  } else if (target.kind === "unassigned") {
    nextLayout.unassigned = [...nextLayout.unassigned, tileId];
  } else if (target.kind === "new-meld") {
    nextLayout.naki = [
      ...nextLayout.naki,
      {
        id: `meld_${Date.now()}`,
        kind: "unknown",
        tiles: [tileId],
        needs_confirmation: true,
      },
    ];
  } else {
    nextLayout.naki = nextLayout.naki.map((meld) =>
      meld.id === target.meldId ? { ...meld, tiles: [...meld.tiles, tileId] } : meld,
    );
  }

  return { ...recognition, layout: nextLayout };
}

function updateTile(
  recognition: RecognitionResult,
  tileId: number,
  tile: TileId,
  onChange: (recognition: RecognitionResult) => void,
) {
  onChange({
    ...recognition,
    detections: recognition.detections.map((candidate) =>
      candidate.id === tileId ? { ...candidate, tile_id: tile } : candidate,
    ),
  });
}

function deleteTile(recognition: RecognitionResult, tileId: number): RecognitionResult {
  return {
    ...recognition,
    detections: recognition.detections.filter((tile) => tile.id !== tileId),
    layout: {
      ...recognition.layout,
      hand: recognition.layout.hand.filter((id) => id !== tileId),
      hora: recognition.layout.hora === tileId ? null : recognition.layout.hora,
      naki: recognition.layout.naki
        .map((meld) => ({ ...meld, tiles: meld.tiles.filter((id) => id !== tileId) }))
        .filter((meld) => meld.tiles.length > 0),
      unassigned: recognition.layout.unassigned.filter((id) => id !== tileId),
    },
  };
}

function tileMap(recognition: RecognitionResult) {
  return new Map(recognition.detections.map((tile) => [tile.id, tile]));
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
