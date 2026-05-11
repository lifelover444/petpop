import { describe, expect, it } from "vitest";
import { idleScene, initialScene, selectScene } from "./sceneEngine";

describe("scene scheduler", () => {
  it("lets interaction override idle", () => {
    const now = 1000;
    const scene = selectScene(initialScene(now), [
      idleScene(now, 0),
      { source: "interaction", state: "waving", timestamp: now },
    ], now);

    expect(scene.state).toBe("waving");
  });

  it("keeps a locked interaction from flickering to movement", () => {
    const current = {
      state: "jumping" as const,
      source: "interaction" as const,
      lockedUntil: 2000,
    };
    const scene = selectScene(current, [
      { source: "movement", state: "running-right", timestamp: 1200 },
    ], 1200);

    expect(scene.state).toBe("jumping");
  });

  it("prioritizes Codex over focus and focus over idle", () => {
    const now = 1000;
    const scene = selectScene(initialScene(now), [
      idleScene(now, 0),
      { source: "focus", state: "running", timestamp: now },
      { source: "codex", state: "review", timestamp: now },
    ], now);

    expect(scene.state).toBe("review");
  });

  it("lets short feedback override Codex", () => {
    const now = 1000;
    const scene = selectScene(initialScene(now), [
      { source: "codex", state: "running", timestamp: now },
      { source: "feedback", state: "jumping", timestamp: now },
    ], now);

    expect(scene.state).toBe("jumping");
  });

  it("lets drag movement override ambient Codex and focus state", () => {
    const now = 1000;
    const scene = selectScene(initialScene(now), [
      idleScene(now, 0),
      { source: "focus", state: "running", timestamp: now },
      { source: "codex", state: "review", timestamp: now },
      { source: "movement", state: "running-left", timestamp: now },
    ], now);

    expect(scene.state).toBe("running-left");
  });

  it("returns to ambient state when drag movement stops", () => {
    const current = {
      state: "running-left" as const,
      source: "movement" as const,
      lockedUntil: 1000,
    };
    const scene = selectScene(current, [
      { source: "codex", state: "review", timestamp: 1100 },
    ], 1100);

    expect(scene.state).toBe("review");
  });

  it("lets drag movement change direction during its animation lock", () => {
    const current = {
      state: "running-left" as const,
      source: "movement" as const,
      lockedUntil: 4000,
    };
    const scene = selectScene(current, [
      {
        source: "movement",
        state: "running-right",
        timestamp: 1200,
        minDurationMs: 3000,
      },
    ], 1200);

    expect(scene.state).toBe("running-right");
    expect(scene.lockedUntil).toBe(4200);
  });

  it("returns to Codex after feedback lock expires", () => {
    const current = {
      state: "jumping" as const,
      source: "feedback" as const,
      lockedUntil: 2000,
    };
    const scene = selectScene(current, [
      { source: "codex", state: "running", timestamp: 2100 },
    ], 2100);

    expect(scene.state).toBe("running");
  });

  it("maps long inactivity to waiting", () => {
    expect(idleScene(0, 60_000).state).toBe("waiting");
    expect(idleScene(0, 59_999).state).toBe("idle");
  });
});
