import { describe, expect, it } from "vitest";
import focusPanelSource from "./FocusPanel.svelte?raw";

describe("focus panel styles", () => {
  it("does not globally disable scrolling for every app window", () => {
    expect(focusPanelSource).not.toContain(":global(html),");
    expect(focusPanelSource).not.toContain(":global(body),");
    expect(focusPanelSource).not.toContain(":global(#app)");
    expect(focusPanelSource).toContain("focus-panel-runtime");
  });
});
