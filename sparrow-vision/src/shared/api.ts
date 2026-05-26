import { invoke } from "@tauri-apps/api/core";
import type {
  AnalyzeHandRequest,
  RecognitionResult,
  ScoringResult,
} from "./types";

export async function recognizeImage(file: File): Promise<RecognitionResult> {
  const buffer = await file.arrayBuffer();
  const imageBytes = Array.from(new Uint8Array(buffer));
  return invoke<RecognitionResult>("recognize_image", { imageBytes });
}

export function analyzeHand(
  request: AnalyzeHandRequest,
): Promise<ScoringResult> {
  return invoke<ScoringResult>("analyze_hand", { request });
}
