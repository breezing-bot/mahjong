import { useMemo, useState } from "react";
import "./App.css";
import { HandEditor } from "./features/hand-editor/HandEditor";
import { RecognitionPreview } from "./features/recognition/RecognitionPreview";
import { ScoringPanel } from "./features/scoring/ScoringPanel";
import { ImageUpload } from "./features/upload/ImageUpload";
import { analyzeHand, recognizeImage } from "./shared/api";
import type {
  AnalyzeHandRequest,
  AnalyzeHandResult,
  MeldKind,
  RecognitionMeld,
  RecognitionResult,
  TileId,
} from "./shared/types";

const DEFAULT_REQUEST: AnalyzeHandRequest = {
  hand: [],
  hora: "1m",
  naki: [],
  zikaze: "east",
  bakaze: "east",
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
  dora: [],
  ura_dora: [],
};

function App() {
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  const [recognition, setRecognition] = useState<RecognitionResult | null>(null);
  const [selectedTileId, setSelectedTileId] = useState<number | null>(null);
  const [request, setRequest] = useState<AnalyzeHandRequest>(DEFAULT_REQUEST);
  const [result, setResult] = useState<AnalyzeHandResult | null>(null);
  const [scoringError, setScoringError] = useState<string | null>(null);
  const [busy, setBusy] = useState<"idle" | "recognizing" | "scoring">("idle");

  const status = useMemo(() => {
    if (busy === "recognizing") return "正在识别照片";
    if (busy === "scoring") return "正在计算点数";
    if (scoringError) return "需要修正输入";
    if (result) return "计算完成";
    return "等待照片或手动录入";
  }, [busy, result, scoringError]);

  async function handleFile(file: File) {
    setBusy("recognizing");
    setResult(null);
    setScoringError(null);
    setRecognition(null);
    setSelectedTileId(null);
    setRequest((current) => ({
      ...current,
      hand: [],
      hora: DEFAULT_REQUEST.hora,
      naki: [],
    }));
    const nextUrl = URL.createObjectURL(file);
    setImageUrl((current) => {
      if (current) URL.revokeObjectURL(current);
      return nextUrl;
    });

    try {
      const nextRecognition = await recognizeImage(file);
      setRecognition(nextRecognition);
      setSelectedTileId(nextRecognition.layout.hora);
      setRequest((current) => requestFromRecognition(current, nextRecognition));
    } catch (error) {
      setRecognition({
        detections: [],
        layout: {
          hand: [],
          hora: null,
          naki: [],
          unassigned: [],
        },
      });
      setScoringError(errorMessage(error));
    } finally {
      setBusy("idle");
    }
  }

  async function handleAnalyze() {
    const validationErrors = recognition ? recognitionValidationErrors(recognition) : [];
    if (validationErrors.length > 0) {
      setResult(null);
      setScoringError(validationErrors.join("\n"));
      return;
    }

    setBusy("scoring");
    setScoringError(null);
    try {
      setResult(await analyzeHand(request));
    } catch (error) {
      setResult(null);
      setScoringError(errorMessage(error));
    } finally {
      setBusy("idle");
    }
  }

  function handleRecognitionChange(nextRecognition: RecognitionResult) {
    setRecognition(nextRecognition);
    setResult(null);
    setScoringError(null);
    setRequest((current) => requestFromRecognition(current, nextRecognition));
  }

  return (
    <main className="app-shell">
      <header className="topbar">
        <div>
          <h1>sparrow-vision</h1>
          <p>{status}</p>
        </div>
        <div className="topbar-stats">
          <span>{request.hand.length} 张闭手牌</span>
          <span>{request.naki.length} 组副露</span>
        </div>
      </header>
      <div className="workspace">
        <div className="left-column">
          <ImageUpload busy={busy === "recognizing"} onFile={handleFile} />
          <RecognitionPreview
            busy={busy === "recognizing"}
            imageUrl={imageUrl}
            recognition={recognition}
            selectedTileId={selectedTileId}
            onTileSelect={setSelectedTileId}
          />
        </div>
        <HandEditor
          recognition={recognition}
          selectedTileId={selectedTileId}
          onRecognitionChange={handleRecognitionChange}
          onSelectedTileChange={setSelectedTileId}
        />
        <ScoringPanel
          busy={busy === "scoring"}
          request={request}
          result={result}
          error={scoringError}
          onAnalyze={handleAnalyze}
          onChange={setRequest}
        />
      </div>
    </main>
  );
}

export default App;

function requestFromRecognition(
  current: AnalyzeHandRequest,
  recognition: RecognitionResult,
): AnalyzeHandRequest {
  const tiles = tileMap(recognition);
  const hand = tileIdsToTileIds(recognition.layout.hand, tiles);
  const hora =
    (recognition.layout.hora !== null
      ? tiles.get(recognition.layout.hora)?.tile_id
      : undefined) ?? current.hora;
  const naki = recognition.layout.naki
    .filter((meld): meld is RecognitionMeld & { kind: MeldKind } => meld.kind !== "unknown")
    .map((meld) => ({
      kind: meld.kind,
      tiles: tileIdsToTileIds(meld.tiles, tiles),
    }));

  return {
    ...current,
    hand,
    hora,
    naki,
  };
}

function recognitionValidationErrors(recognition: RecognitionResult): string[] {
  const errors: string[] = [];
  if (recognition.layout.hora === null) {
    errors.push("请先选择和了牌。");
  }
  if (recognition.layout.naki.some((meld) => meld.kind === "unknown")) {
    errors.push("请先确认所有副露类型。");
  }
  return errors;
}

function tileMap(recognition: RecognitionResult) {
  return new Map(recognition.detections.map((tile) => [tile.id, tile]));
}

function tileIdsToTileIds(ids: number[], tiles: Map<number, { tile_id: TileId }>): TileId[] {
  return ids.map((id) => tiles.get(id)?.tile_id).filter(Boolean) as TileId[];
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
