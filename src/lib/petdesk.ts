import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { PetAnimationState } from "./animations";
import sakikoSpriteUrl from "../../sakiko/spritesheet.webp?url";
import taffySpriteUrl from "../../taffy/spritesheet.webp?url";

export interface PetInfo {
  id: string;
  displayName: string;
  description: string;
  spritesheetPath: string;
  sourceKind: string;
  sourceUrl?: string | null;
  scale: number;
}

export interface RuntimeState {
  activePetId?: string | null;
  scene: PetAnimationState;
  scale: number;
}

export const isTauri = () => Boolean(window.__TAURI_INTERNALS__);

const demoPets: PetInfo[] = [
  {
    id: "taffy",
    displayName: "Taffy",
    description:
      "A cute chibi digital pet inspired by the VTuber Ace Taffy, a pink-haired detective-inventor.",
    spritesheetPath: taffySpriteUrl,
    sourceKind: "demo",
    scale: 1,
  },
  {
    id: "sakiko",
    displayName: "Sakiko",
    description:
      "A tiny chibi digital pet with pale blue twin-tail hair and a burgundy gothic idol outfit.",
    spritesheetPath: sakikoSpriteUrl,
    sourceKind: "demo",
    scale: 1,
  },
];

let browserRuntime: RuntimeState = {
  activePetId: "taffy",
  scene: "idle",
  scale: 1,
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
    return demoPets;
  }

  return invoke<PetInfo[]>("list_pets");
}

export async function importPetFromPath(path: string): Promise<PetInfo> {
  if (!isTauri()) {
    throw new Error("Local import requires the desktop app.");
  }

  const pet = await invoke<PetInfo>("import_pet_from_path", { path });
  spriteDataUrlCache.delete(pet.id);
  return pet;
}

export async function importPetdex(input: string): Promise<PetInfo> {
  if (!isTauri()) {
    throw new Error("PetDex import requires the desktop app.");
  }

  const pet = await invoke<PetInfo>("import_petdex", { input });
  spriteDataUrlCache.delete(pet.id);
  return pet;
}

export async function scanCodexPets(): Promise<PetInfo[]> {
  if (!isTauri()) {
    return demoPets;
  }

  const pets = await invoke<PetInfo[]>("scan_codex_pets");
  spriteDataUrlCache.clear();
  return pets;
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
            { name: "Pet package", extensions: ["zip"] },
            { name: "All files", extensions: ["*"] },
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
  if (!isTauri()) {
    browserRuntime = { ...browserRuntime, scale };
    return browserRuntime;
  }

  return invoke<RuntimeState>("set_scale", { scale });
}
