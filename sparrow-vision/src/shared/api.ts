import { invoke } from "@tauri-apps/api/core";
import type {
  AnalyzeHandRequestV1,
  RecognitionResultV1,
  ScoringResultV1,
} from "./types";

export async function recognizeImage(file: File): Promise<RecognitionResultV1> {
  const buffer = await file.arrayBuffer();
  const imageBytes = Array.from(new Uint8Array(buffer));
  return invoke<RecognitionResultV1>("recognize_image", { imageBytes });
}

export function analyzeHand(
  request: AnalyzeHandRequestV1,
): Promise<ScoringResultV1> {
  return invoke<ScoringResultV1>("analyze_hand", { request });
}
