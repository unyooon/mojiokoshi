/**
 * Whisper model commands.
 *
 * These wrappers will be replaced once tauri-specta regenerates bindings.ts
 * with the new Rust commands. Until then, they call TAURI_INVOKE directly
 * following the same Result pattern used by the generated bindings.
 */
import { invoke } from "@tauri-apps/api/core";
import type { Result, AppError } from "@/bindings";

export type WhisperModelStatus =
  | "NotDownloaded"
  | { Downloading: { progress: number } }
  | { Ready: { path: string } };

export async function getWhisperModelStatus(): Promise<Result<WhisperModelStatus, AppError>> {
  try {
    return {
      status: "ok",
      data: await invoke<WhisperModelStatus>("get_whisper_model_status"),
    };
  } catch (e: unknown) {
    if (e instanceof Error) throw e;
    return { status: "error", error: e as AppError };
  }
}

export async function downloadWhisperModel(model: string): Promise<Result<string, AppError>> {
  try {
    return {
      status: "ok",
      data: await invoke<string>("download_whisper_model", { model }),
    };
  } catch (e: unknown) {
    if (e instanceof Error) throw e;
    return { status: "error", error: e as AppError };
  }
}
