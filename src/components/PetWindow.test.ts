import { describe, expect, it } from "vitest";
import petWindowSource from "./PetWindow.svelte?raw";

describe("pet window source", () => {
  it("does not update playback state inside the pet-loading effect", () => {
    const petLoadingEffect = petWindowSource.match(
      /\$effect\(\(\) => \{\s+const petSignature = activePetSignature;[\s\S]*?getPetSpriteUrl\(pet\)/,
    )?.[0];

    expect(petLoadingEffect).toBeTruthy();
    expect(petLoadingEffect).not.toContain("scenePlaybackKey +=");
    expect(petLoadingEffect).toContain("untrack(() => activePet)");
  });

  it("does not force idle immediately when drag ends", () => {
    const endDragFunction = petWindowSource.match(
      /async function endDrag\(\) \{[\s\S]*?\n  \}/,
    )?.[0];

    expect(endDragFunction).toBeTruthy();
    expect(endDragFunction).not.toContain('commitScene("idle")');
    expect(endDragFunction).not.toContain("scheduledScene = initialScene");
  });

  it("locks click-style actions for three full animation cycles", () => {
    const triggerActionFunction = petWindowSource.match(
      /async function triggerAction[\s\S]*?\n  \}/,
    )?.[0];

    expect(triggerActionFunction).toBeTruthy();
    expect(triggerActionFunction).toContain("repeatedAnimationDurationMs(resolvedScene, 3)");
  });

  it("opens the compact focus bubble near the right-click position", () => {
    expect(petWindowSource).toContain("const FOCUS_PANEL_WIDTH = 524");
    expect(petWindowSource).toContain("const FOCUS_PANEL_HEIGHT = 148");
    expect(petWindowSource).toContain("oncontextmenu={openFocusPanel}");
  });
});
