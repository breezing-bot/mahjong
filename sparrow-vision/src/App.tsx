import { useMemo, useState } from "react";
import "./App.css";
import { HandEditor } from "./features/hand-editor/HandEditor";
import { RecognitionPreview } from "./features/recognition/RecognitionPreview";
import { ScoringPanel } from "./features/scoring/ScoringPanel";
import { ImageUpload } from "./features/upload/ImageUpload";
import { analyzeHand, recognizeImage } from "./shared/api";
import type {
  AnalyzeHandRequestV1,
  MeldInput,
  RecognitionResultV1,
  ScoringResultV1,
  TileId,
} from "./shared/types";

const DEFAULT_REQUEST: AnalyzeHandRequestV1 = {
  hand_tiles: [],
  winning_tile: null,
  melds: [],
  self_wind: "east",
  round_wind: "east",
  win_method: "ron",
  riichi: "none",
  special_win: {
    ippatsu: false,
    chankan: false,
    rinshan: false,
    haitei: false,
    hotei: false,
    first_turn_tsumo: false,
  },
  honba: 0,
  dora_indicators: [],
  ura_dora_indicators: [],
};

function App() {
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  const [recognition, setRecognition] = useState<RecognitionResultV1 | null>(null);
  const [request, setRequest] = useState<AnalyzeHandRequestV1>(DEFAULT_REQUEST);
  const [pendingTiles, setPendingTiles] = useState<TileId[]>([]);
  const [result, setResult] = useState<ScoringResultV1 | null>(null);
  const [busy, setBusy] = useState<"idle" | "recognizing" | "scoring">("idle");

  const status = useMemo(() => {
    if (busy === "recognizing") return "正在识别照片";
    if (busy === "scoring") return "正在计算番符";
    if (result?.errors.length) return "需要修正输入";
    if (result?.is_win) return "计算完成";
    return "等待照片或手动录入";
  }, [busy, result]);

  async function handleFile(file: File) {
    setBusy("recognizing");
    setResult(null);
    const nextUrl = URL.createObjectURL(file);
    setImageUrl((current) => {
      if (current) URL.revokeObjectURL(current);
      return nextUrl;
    });

    try {
      const nextRecognition = await recognizeImage(file);
      setRecognition(nextRecognition);
      setRequest((current) => ({
        ...current,
        hand_tiles: nextRecognition.hand_tiles,
        winning_tile:
          nextRecognition.hand_tiles[nextRecognition.hand_tiles.length - 1] ??
          current.winning_tile,
      }));
      const handSet = new Set(nextRecognition.hand_tiles);
      setPendingTiles(
        nextRecognition.detections
          .map((detection) => detection.tile_id)
          .filter((tile) => !handSet.has(tile)),
      );
    } catch (error) {
      setRecognition({
        detections: [],
        hand_tiles: [],
        quality_flags: ["model_unavailable"],
        diagnostics: {
          raw_detection_count: 0,
          kept_detection_count: 0,
          dropped_low_confidence_count: 0,
          dropped_unknown_class_count: 0,
          dropped_duplicate_count: 0,
        },
      });
      setResult({
        is_win: false,
        yaku: [],
        han: 0,
        fu: 0,
        score_breakdown: null,
        waits: [],
        errors: [error instanceof Error ? error.message : String(error)],
      });
    } finally {
      setBusy("idle");
    }
  }

  async function handleAnalyze() {
    setBusy("scoring");
    try {
      setResult(await analyzeHand(request));
    } catch (error) {
      setResult({
        is_win: false,
        yaku: [],
        han: 0,
        fu: 0,
        score_breakdown: null,
        waits: [],
        errors: [error instanceof Error ? error.message : String(error)],
      });
    } finally {
      setBusy("idle");
    }
  }

  return (
    <main className="app-shell">
      <header className="topbar">
        <div>
          <h1>sparrow-vision</h1>
          <p>{status}</p>
        </div>
        <div className="topbar-stats">
          <span>{request.hand_tiles.length} 手牌</span>
          <span>{request.melds.length} 副露</span>
        </div>
      </header>
      <div className="workspace">
        <div className="left-column">
          <ImageUpload
            busy={busy === "recognizing"}
            imageUrl={imageUrl}
            onFile={handleFile}
          />
          <RecognitionPreview imageUrl={imageUrl} recognition={recognition} />
        </div>
        <HandEditor
          handTiles={request.hand_tiles}
          melds={request.melds}
          pendingTiles={pendingTiles}
          winningTile={request.winning_tile}
          onHandTilesChange={(hand_tiles) =>
            setRequest((current) => ({ ...current, hand_tiles }))
          }
          onMeldsChange={(melds: MeldInput[]) =>
            setRequest((current) => ({ ...current, melds }))
          }
          onPendingTilesChange={setPendingTiles}
          onWinningTileChange={(winning_tile) =>
            setRequest((current) => ({ ...current, winning_tile }))
          }
        />
        <ScoringPanel
          busy={busy === "scoring"}
          request={request}
          result={result}
          onAnalyze={handleAnalyze}
          onChange={setRequest}
        />
      </div>
    </main>
  );
}

export default App;
