import type { PetActionEvent } from "./actions";

export type DragActionEvent = Extract<PetActionEvent, "drag-left" | "drag-right">;
export type ClickActionEvent = Extract<PetActionEvent, "click" | "double-click">;

export const DRAG_THRESHOLD_PX = 5;
export const DRAG_DIRECTION_MIN_DELTA_PX = 1;
export const DOUBLE_CLICK_MS = 260;

export function hasDragMovement(
  startX: number,
  startY: number,
  currentX: number,
  currentY: number,
  thresholdPx = DRAG_THRESHOLD_PX,
) {
  return Math.hypot(currentX - startX, currentY - startY) >= thresholdPx;
}

export function dragEventFromDelta(
  deltaX: number,
  previousEvent: DragActionEvent | null,
  minDeltaPx = DRAG_DIRECTION_MIN_DELTA_PX,
): DragActionEvent | null {
  if (Math.abs(deltaX) < minDeltaPx) {
    return previousEvent;
  }

  return deltaX > 0 ? "drag-right" : "drag-left";
}

export function nextClickAction(hasQueuedClick: boolean): ClickActionEvent {
  return hasQueuedClick ? "double-click" : "click";
}
