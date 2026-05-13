import { describe, expect, it } from "vitest";
import defaultCapability from "../src-tauri/capabilities/default.json";
import tauriConfig from "../src-tauri/tauri.conf.json";

describe("tauri focus panel window", () => {
  it("uses a transparent undecorated surface so rounded UI has no outer window fill", () => {
    const focusWindow = tauriConfig.app.windows.find(
      (window) => window.label === "focus-panel",
    );

    expect(focusWindow).toMatchObject({
      decorations: false,
      transparent: true,
      backgroundColor: "#00000000",
      shadow: false,
    });
  });

  it("sizes the focus panel wide enough for timer, settings, and actions", () => {
    const focusWindow = tauriConfig.app.windows.find(
      (window) => window.label === "focus-panel",
    );

    expect(focusWindow?.width).toBeGreaterThanOrEqual(800);
    expect(focusWindow?.height).toBeGreaterThanOrEqual(154);
  });

  it("keeps the main window wide enough for the three-column control layout", () => {
    const mainWindow = tauriConfig.app.windows.find(
      (window) => window.label === "main",
    );

    expect(mainWindow?.minWidth).toBeGreaterThanOrEqual(900);
  });

  it("allows the JS window APIs used by the pet focus launcher", () => {
    expect(defaultCapability.permissions).toEqual(
      expect.arrayContaining([
        "core:event:default",
        "core:window:allow-inner-size",
        "core:window:allow-scale-factor",
        "core:window:allow-set-size",
        "core:window:allow-unminimize",
      ]),
    );
  });
});
