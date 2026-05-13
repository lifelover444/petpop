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
    expect(petWindowSource).toContain("const FOCUS_LAUNCHER_WIDTH = 132");
    expect(petWindowSource).toContain("const FOCUS_LAUNCHER_HEIGHT = 46");
    expect(petWindowSource).toContain("const FOCUS_PANEL_WIDTH = 520");
    expect(petWindowSource).toContain("const FOCUS_PANEL_HEIGHT = 132");
    expect(petWindowSource).toContain("focusPanel.setSize");
    expect(petWindowSource).toContain("new LogicalSize(FOCUS_LAUNCHER_WIDTH, FOCUS_LAUNCHER_HEIGHT)");
    expect(petWindowSource).toContain("new LogicalPosition(position.x, position.y)");
    expect(petWindowSource).toContain("oncontextmenu={openFocusPanel}");
  });

  it("keeps the interactive hit target as large as the native pet window", () => {
    expect(petWindowSource).toContain("width: 192px");
    expect(petWindowSource).toContain("height: 208px");
    expect(petWindowSource).toContain("display: grid");
    expect(petWindowSource).toContain("place-items: start");
  });

  it("forces the focus panel back to launcher mode after showing it", () => {
    const showLauncherFunction = petWindowSource.match(
      /async function showFocusPanelLauncher[\s\S]*?\n  \}/,
    )?.[0];

    expect(showLauncherFunction).toBeTruthy();
    expect(showLauncherFunction).toContain("await focusPanel.hide()");
    expect(showLauncherFunction).toContain("await focusPanel.show()");
    expect(showLauncherFunction).toContain("await focusPanel.setFocus()");
    expect(showLauncherFunction).toContain(
      'await focusPanel.emitTo("focus-panel", "focus-panel:launcher")',
    );
    expect(showLauncherFunction).toContain("window.setTimeout");
  });
});
