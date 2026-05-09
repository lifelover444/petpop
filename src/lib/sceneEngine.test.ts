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

  it("maps long inactivity to waiting", () => {
    expect(idleScene(0, 60_000).state).toBe("waiting");
    expect(idleScene(0, 59_999).state).toBe("idle");
  });
});
