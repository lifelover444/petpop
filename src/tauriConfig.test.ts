import { describe, expect, it } from "vitest";
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
});
