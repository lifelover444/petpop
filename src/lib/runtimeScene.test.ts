import { describe, expect, it } from "vitest";
import { repeatedAnimationDurationMs } from "./animations";
import { actionSceneEvent } from "./runtimeScene";

describe("runtime action scene events", () => {
  it("keeps transient actions visible for three full animation cycles by default", () => {
    const event = actionSceneEvent(null, "click", "interaction", 1000);

    expect(event.state).toBe("waving");
    expect(event.minDurationMs).toBe(repeatedAnimationDurationMs("waving", 3));
  });

  it("keeps double-click visible for three full animation cycles by default", () => {
    const event = actionSceneEvent(null, "double-click", "interaction", 1000);

    expect(event.state).toBe("jumping");
    expect(event.minDurationMs).toBe(repeatedAnimationDurationMs("jumping", 3));
  });

  it("keeps short feedback visible for three full animation cycles by default", () => {
    const event = actionSceneEvent(null, "codex-error", "feedback", 1000);

    expect(event.state).toBe("failed");
    expect(event.minDurationMs).toBe(repeatedAnimationDurationMs("failed", 3));
  });
});
