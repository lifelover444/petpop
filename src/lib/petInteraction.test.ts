import { describe, expect, it } from "vitest";
import {
  dragEventFromDelta,
  hasDragMovement,
  nextClickAction,
} from "./petInteraction";

describe("Codex-style pet interaction rules", () => {
  it("starts dragging only after a small pointer movement threshold", () => {
    expect(hasDragMovement(20, 20, 23, 23)).toBe(false);
    expect(hasDragMovement(20, 20, 25, 20)).toBe(true);
  });

  it("maps horizontal drag direction to fixed Codex movement actions", () => {
    expect(dragEventFromDelta(2, null)).toBe("drag-right");
    expect(dragEventFromDelta(-2, null)).toBe("drag-left");
  });

  it("keeps the previous drag direction for jitter-sized samples", () => {
    expect(dragEventFromDelta(0, "drag-left")).toBe("drag-left");
    expect(dragEventFromDelta(0.5, "drag-right")).toBe("drag-right");
    expect(dragEventFromDelta(0, null)).toBeNull();
  });

  it("turns a queued click into a double-click on the second release", () => {
    expect(nextClickAction(false)).toBe("click");
    expect(nextClickAction(true)).toBe("double-click");
  });
});
