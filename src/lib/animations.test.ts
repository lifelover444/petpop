import { describe, expect, it } from "vitest";
import {
  ANIMATION_ROWS,
  ATLAS_HEIGHT,
  ATLAS_WIDTH,
  CELL_HEIGHT,
  CELL_WIDTH,
  animationCycleDurationMs,
  repeatedAnimationDurationMs,
  framePosition,
  nextFrameDelay,
} from "./animations";

describe("Codex animation table", () => {
  it("matches the Codex atlas dimensions", () => {
    expect(ATLAS_WIDTH).toBe(1536);
    expect(ATLAS_HEIGHT).toBe(1872);
  });

  it("keeps every row within the atlas", () => {
    for (const row of ANIMATION_ROWS) {
      const lastFrame = framePosition(row.state, row.frames - 1);
      expect(lastFrame.x + CELL_WIDTH).toBeLessThanOrEqual(ATLAS_WIDTH);
      expect(lastFrame.y + CELL_HEIGHT).toBeLessThanOrEqual(ATLAS_HEIGHT);
      expect(row.durations).toHaveLength(row.frames);
    }
  });

  it("uses the documented review frame delay", () => {
    expect(nextFrameDelay("review", 5)).toBe(280);
  });

  it("calculates full-row animation duration for repeated actions", () => {
    expect(animationCycleDurationMs("running-left")).toBe(1060);
    expect(repeatedAnimationDurationMs("running-left", 3)).toBe(3180);
    expect(repeatedAnimationDurationMs("waving", 3)).toBe(2100);
    expect(repeatedAnimationDurationMs("jumping", 3)).toBe(2520);
  });
});
