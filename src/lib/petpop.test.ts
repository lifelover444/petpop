import { describe, expect, it } from "vitest";
import petpopSource from "./petpop.ts?raw";

describe("petpop Tauri wrappers", () => {
  it("removes an imported pet copy through the Rust command", () => {
    expect(petpopSource).toContain("export async function removePet");
    expect(petpopSource).toContain('invoke<RuntimeState>("remove_pet"');
    expect(petpopSource).toContain("spriteDataUrlCache.delete(petId)");
  });
});
