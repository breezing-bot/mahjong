import type {
  AnalyzeHandRequest,
  AnalyzeHandResult,
  RiichiInput,
  TileId,
  WindInput,
  WinMethodInput,
} from "../../shared/types";
import { TileBadge } from "../../shared/TileBadge";
import { SCORING_TILE_IDS, tileLabel, WIND_OPTIONS } from "../../shared/tiles";

interface ScoringPanelProps {
  request: AnalyzeHandRequest;
  result: AnalyzeHandResult | null;
  error: string | null;
  busy: boolean;
  onChange: (request: AnalyzeHandRequest) => void;
  onAnalyze: () => void;
}

export function ScoringPanel({
  request,
  result,
  error,
  busy,
  onChange,
  onAnalyze,
}: ScoringPanelProps) {
  return (
    <section className="panel scoring-panel">
      <div className="panel-heading">
        <h2>算点</h2>
        <button type="button" disabled={busy} onClick={onAnalyze}>
          {busy ? "计算中" : "计算"}
        </button>
      </div>
      <div className="settings-section">
        <h3>基础设置</h3>
        <div className="settings-grid">
          <label>
            场风
            <select
              value={request.bakaze}
              onChange={(event) =>
                onChange({ ...request, bakaze: event.currentTarget.value as WindInput })
              }
            >
              {WIND_OPTIONS.map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </label>
          <label>
            自风
            <select
              value={request.zikaze}
              onChange={(event) =>
                onChange({ ...request, zikaze: event.currentTarget.value as WindInput })
              }
            >
              {WIND_OPTIONS.map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </label>
          <label>
            和牌方式
            <select
              value={request.win_method}
              onChange={(event) =>
                onChange({
                  ...request,
                  win_method: event.currentTarget.value as WinMethodInput,
                })
              }
            >
              <option value="ron">荣和</option>
              <option value="tsumo">自摸</option>
            </select>
          </label>
          <label>
            立直
            <select
              value={request.riichi}
              onChange={(event) =>
                onChange({ ...request, riichi: event.currentTarget.value as RiichiInput })
              }
            >
              <option value="none">无</option>
              <option value="riichi">立直</option>
              <option value="double_riichi">双立直</option>
            </select>
          </label>
          <label>
            本场
            <input
              min={0}
              type="number"
              value={request.honba}
              onChange={(event) =>
                onChange({ ...request, honba: Number(event.currentTarget.value) })
              }
            />
          </label>
        </div>
      </div>
      <div className="settings-section special-win-section">
        <h3>特殊役状况</h3>
        <div className="toggle-grid">
          {[
            ["ippatsu", "一发"],
            ["chankan", "抢杠"],
            ["rinshan", "岭上"],
            ["haitei", "海底"],
            ["hotei", "河底"],
            ["first_turn_tsumo", "首巡自摸"],
          ].map(([key, label]) => (
            <label className="toggle" key={key}>
              <input
                checked={Boolean(request.special_win[key as keyof typeof request.special_win])}
                type="checkbox"
                onChange={(event) =>
                  onChange({
                    ...request,
                    special_win: {
                      ...request.special_win,
                      [key]: event.currentTarget.checked,
                    },
                  })
                }
              />
              {label}
            </label>
          ))}
        </div>
      </div>
      <IndicatorEditor
        label="宝牌指示牌"
        tiles={request.dora}
        onChange={(tiles) => onChange({ ...request, dora: tiles })}
      />
      <IndicatorEditor
        label="里宝牌指示牌"
        tiles={request.ura_dora}
        onChange={(tiles) => onChange({ ...request, ura_dora: tiles })}
      />
      <ResultView result={result} error={error} />
    </section>
  );
}

function IndicatorEditor({
  label,
  tiles,
  onChange,
}: {
  label: string;
  tiles: TileId[];
  onChange: (tiles: TileId[]) => void;
}) {
  return (
    <div className="indicator-editor">
      <div className="rack-heading">
        <h3>{label}</h3>
        <select
          aria-label={label}
          defaultValue=""
          onChange={(event) => {
            const tile = event.currentTarget.value as TileId;
            if (tile) {
              onChange([...tiles, tile]);
              event.currentTarget.value = "";
            }
          }}
        >
          <option value="" disabled>
            添加
          </option>
          {SCORING_TILE_IDS.map((tile) => (
            <option key={tile} value={tile}>
              {tileLabel(tile)}
            </option>
          ))}
        </select>
      </div>
      <div className="tile-row compact">
        {tiles.map((tile, index) => (
          <TileBadge
            key={`${label}-${tile}-${index}`}
            tile={tile}
            onClick={() => onChange(tiles.filter((_, i) => i !== index))}
          />
        ))}
      </div>
    </div>
  );
}

function ResultView({
  result,
  error,
}: {
  result: AnalyzeHandResult | null;
  error: string | null;
}) {
  if (error) {
    return (
      <div className="result error-result">
        {error.split("\n").map((line) => (
          <p key={line}>{line}</p>
        ))}
      </div>
    );
  }
  if (!result) {
    return <div className="result-empty">等待计算</div>;
  }

  return (
    <div className="result">
      <div className="score-line">
        <strong>{result.han} 番</strong>
        <strong>{result.fu} 符</strong>
      </div>
      {result.points.kind === "ron" ? (
        <p>荣和 {result.points.points} 点</p>
      ) : (
        <p>
          自摸 庄家 {result.points.dealer} / 闲家 {result.points.non_dealer}
        </p>
      )}
      <div className="yaku-list">
        {result.yaku.map((yaku, index) => (
          <span key={`${yaku.name}-${index}`}>
            {yaku.name} {yaku.han}
          </span>
        ))}
      </div>
    </div>
  );
}
