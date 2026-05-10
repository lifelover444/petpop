import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { DEFAULT_ACTION_MAP, type PetActionMap } from "./actions";
import type { PetAnimationState } from "./animations";

export interface PetInfo {
  id: string;
  displayName: string;
  description: string;
  spritesheetPath: string;
  sourceKind: string;
  sourceUrl?: string | null;
  scale: number;
  actionMap: PetActionMap;
}

export interface RuntimeState {
  activePetId?: string | null;
  scene: PetAnimationState;
  scale: number;
  focusState: FocusState;
  codexActivity: CodexActivity;
  codexActivityError?: string | null;
}

export interface PetWindowPosition {
  x: number;
  y: number;
}

export type FocusMode = "idle" | "focus" | "break";
export type FocusStatus = "idle" | "running" | "paused" | "complete";
export type CodexActivityStatus =
  | "idle"
  | "running"
  | "waiting"
  | "review"
  | "success"
  | "error";

export interface FocusState {
  mode: FocusMode;
  status: FocusStatus;
  lastEvent?: string | null;
  remainingMs?: number | null;
  endsAt?: number | null;
  updatedAt: number;
}

export interface CodexActivity {
  status: CodexActivityStatus;
  message?: string | null;
  updatedAt: number;
}

export interface AppSettings {
  focusMinutes: number;
  breakMinutes: number;
}

export const isTauri = () => Boolean(window.__TAURI_INTERNALS__);

export function sourceKindLabel(sourceKind: string) {
  switch (sourceKind) {
    case "local":
      return "本地导入";
    case "codex":
      return "Codex";
    case "petdex":
      return "PetDex";
    case "browser":
      return "浏览器预览";
    case "unknown":
      return "未知来源";
    default:
      return sourceKind;
  }
}

let browserRuntime: RuntimeState = {
  activePetId: null,
  scene: "idle",
  scale: 0.5,
  focusState: {
    mode: "idle",
    status: "idle",
    lastEvent: null,
    remainingMs: null,
    endsAt: null,
    updatedAt: Date.now(),
  },
  codexActivity: {
    status: "idle",
    message: null,
    updatedAt: 0,
  },
  codexActivityError: null,
};
const spriteDataUrlCache = new Map<string, string>();

export function spriteUrl(path: string) {
  return isTauri() ? convertFileSrc(path) : path;
}

export async function getPetSpriteUrl(pet: PetInfo): Promise<string> {
  if (!isTauri()) {
    return pet.spritesheetPath;
  }

  const cached = spriteDataUrlCache.get(pet.id);
  if (cached) {
    return cached;
  }

  const url = await invoke<string>("get_pet_sprite_data_url", { petId: pet.id });
  spriteDataUrlCache.set(pet.id, url);
  return url;
}

export async function listPets(): Promise<PetInfo[]> {
  if (!isTauri()) {
    return [];
  }

  return invoke<PetInfo[]>("list_pets");
}

export async function importPetFromPath(path: string): Promise<PetInfo> {
  if (!isTauri()) {
    throw new Error("本地导入需要在桌面应用中使用。");
  }

  const pet = await invoke<PetInfo>("import_pet_from_path", { path });
  spriteDataUrlCache.delete(pet.id);
  return pet;
}

export async function importPetdex(input: string): Promise<PetInfo> {
  if (!isTauri()) {
    throw new Error("PetDex 导入需要在桌面应用中使用。");
  }

  const pet = await invoke<PetInfo>("import_petdex", { input });
  spriteDataUrlCache.delete(pet.id);
  return pet;
}

export async function scanCodexPets(): Promise<PetInfo[]> {
  if (!isTauri()) {
    return [];
  }

  const pets = await invoke<PetInfo[]>("scan_codex_pets");
  spriteDataUrlCache.clear();
  return pets;
}

export async function setPetActionMap(
  petId: string,
  actionMap: PetActionMap,
): Promise<PetInfo> {
  if (!isTauri()) {
    return {
      id: petId,
      displayName: petId,
      description: "",
      spritesheetPath: "",
      sourceKind: "browser",
      sourceUrl: null,
      scale: 0.5,
      actionMap: { ...DEFAULT_ACTION_MAP, ...actionMap },
    };
  }

  return invoke<PetInfo>("set_pet_action_map", { petId, actionMap });
}

export async function chooseImportPath(
  kind: "file" | "folder",
): Promise<string | null> {
  if (!isTauri()) {
    return null;
  }

  const selected = await open({
    multiple: false,
    directory: kind === "folder",
    filters:
      kind === "file"
        ? [
            { name: "宠物包", extensions: ["zip"] },
            { name: "所有文件", extensions: ["*"] },
          ]
        : undefined,
  });

  if (Array.isArray(selected)) {
    return selected[0] ?? null;
  }

  return selected;
}

export async function getRuntimeState(): Promise<RuntimeState> {
  if (!isTauri()) {
    return browserRuntime;
  }

  return invoke<RuntimeState>("get_runtime_state");
}

export async function setActivePet(petId: string): Promise<RuntimeState> {
  if (!isTauri()) {
    browserRuntime = { ...browserRuntime, activePetId: petId };
    return browserRuntime;
  }

  return invoke<RuntimeState>("set_active_pet", { petId });
}

export async function setScene(
  scene: PetAnimationState,
): Promise<RuntimeState> {
  if (!isTauri()) {
    browserRuntime = { ...browserRuntime, scene };
    return browserRuntime;
  }

  return invoke<RuntimeState>("set_scene", { scene });
}

export async function setScale(scale: number): Promise<RuntimeState> {
  const nextScale = Math.max(0.1, Math.min(1, scale));

  if (!isTauri()) {
    browserRuntime = { ...browserRuntime, scale: nextScale };
    return browserRuntime;
  }

  return invoke<RuntimeState>("set_scale", { scale: nextScale });
}

export async function getAppSettings(): Promise<AppSettings> {
  if (!isTauri()) {
    return { focusMinutes: 25, breakMinutes: 5 };
  }

  return invoke<AppSettings>("get_app_settings");
}

export async function setAppSettings(
  settings: AppSettings,
): Promise<AppSettings> {
  const nextSettings = {
    focusMinutes: Math.max(1, Math.min(180, Math.round(settings.focusMinutes))),
    breakMinutes: Math.max(1, Math.min(60, Math.round(settings.breakMinutes))),
  };

  if (!isTauri()) {
    return nextSettings;
  }

  return invoke<AppSettings>("set_app_settings", nextSettings);
}

export async function setFocusState(
  focusState: Omit<FocusState, "updatedAt">,
): Promise<RuntimeState> {
  if (!isTauri()) {
    browserRuntime = {
      ...browserRuntime,
      focusState: { ...focusState, updatedAt: Date.now() },
    };
    return browserRuntime;
  }

  return invoke<RuntimeState>("set_focus_state", focusState);
}

export async function getPetWindowPosition(): Promise<PetWindowPosition> {
  if (!isTauri()) {
    return { x: 1200, y: 580 };
  }

  return invoke<PetWindowPosition>("get_pet_window_position");
}

export async function setPetWindowPosition(
  position: PetWindowPosition,
): Promise<void> {
  if (!isTauri()) {
    return;
  }

  await invoke("set_pet_window_position", { x: position.x, y: position.y });
}
