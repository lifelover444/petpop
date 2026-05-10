import { describe, expect, it } from "vitest";
import {
  ACTION_EVENTS,
  DEFAULT_ACTION_MAP,
  normalizeActionMap,
  resolvePetAction,
} from "./actions";

describe("pet action mapping", () => {
  it("covers every desktop action with a default animation", () => {
    for (const { event } of ACTION_EVENTS) {
      expect(DEFAULT_ACTION_MAP[event]).toBeTruthy();
    }
  });

  it("adds Codex and focus events without adding animation states", () => {
    expect(resolvePetAction(undefined, "codex-running")).toBe("running");
    expect(resolvePetAction(undefined, "codex-review")).toBe("review");
    expect(resolvePetAction(undefined, "focus-start")).toBe("running");
    expect(resolvePetAction(undefined, "focus-complete")).toBe("jumping");
    expect(resolvePetAction(undefined, "break-start")).toBe("waving");
  });

  it("lets pets override known actions", () => {
    expect(resolvePetAction({ click: "jumping" }, "click")).toBe("jumping");
  });

  it("falls back for unknown action keys and invalid animation states", () => {
    const normalized = normalizeActionMap({
      click: "failed",
      "drag-left": "not-a-state",
      "not-an-event": "jumping",
    } as never);

    expect(normalized.click).toBe("failed");
    expect(normalized["drag-left"]).toBe(DEFAULT_ACTION_MAP["drag-left"]);
    expect(Object.keys(normalized)).not.toContain("not-an-event");
  });
});
