import {
  resolvePetAction,
  type PetActionEvent,
  type PetActionMap,
} from "./actions";
import { idleScene, type SceneEvent, type SceneSource } from "./sceneEngine";
import type { CodexActivityStatus, FocusState, RuntimeState } from "./petpop";

const FEEDBACK_VISIBLE_MS = 1400;

export function actionSceneEvent(
  actionMap: PetActionMap | undefined | null,
  event: PetActionEvent,
  source: SceneSource,
  timestamp: number,
  minDurationMs?: number,
): SceneEvent {
  return {
    source,
    state: resolvePetAction(actionMap, event),
    timestamp,
    minDurationMs,
  };
}

export function runtimeSceneEvents(
  runtime: RuntimeState,
  actionMap: PetActionMap | undefined | null,
  now: number,
  inactiveMs: number,
) {
  const events: SceneEvent[] = [];
  const codexEvent = codexSceneEvent(runtime, actionMap, now);
  const focusEvent = focusSceneEvent(runtime.focusState, actionMap, now);

  if (codexEvent) {
    events.push(codexEvent);
  }
  if (focusEvent) {
    events.push(focusEvent);
  }

  events.push(idleScene(now, inactiveMs));
  return events;
}

function codexSceneEvent(
  runtime: RuntimeState,
  actionMap: PetActionMap | undefined | null,
  now: number,
) {
  const { status, updatedAt } = runtime.codexActivity;
  if (status === "idle") {
    return null;
  }

  const event = codexActionEvent(status);
  const isFeedback = status === "success" || status === "error";
  if (isFeedback && now - updatedAt > FEEDBACK_VISIBLE_MS) {
    return null;
  }

  return actionSceneEvent(
    actionMap,
    event,
    isFeedback ? "feedback" : "codex",
    updatedAt || now,
  );
}

function codexActionEvent(status: Exclude<CodexActivityStatus, "idle">) {
  switch (status) {
    case "running":
      return "codex-running";
    case "waiting":
      return "codex-waiting";
    case "review":
      return "codex-review";
    case "success":
      return "codex-success";
    case "error":
      return "codex-error";
  }
}

function focusSceneEvent(
  focusState: FocusState,
  actionMap: PetActionMap | undefined | null,
  now: number,
) {
  const freshEvent =
    focusState.lastEvent && now - focusState.updatedAt <= FEEDBACK_VISIBLE_MS
      ? (focusState.lastEvent as PetActionEvent)
      : null;

  if (freshEvent) {
    return actionSceneEvent(
      actionMap,
      freshEvent,
      focusFeedbackEvent(freshEvent) ? "feedback" : "focus",
      focusState.updatedAt || now,
    );
  }

  if (focusState.mode === "idle" || focusState.status === "idle") {
    return null;
  }

  const event = focusActionEvent(focusState);
  if (!event) {
    return null;
  }

  const isFeedback = focusState.status === "complete";
  if (isFeedback && now - focusState.updatedAt > FEEDBACK_VISIBLE_MS) {
    return null;
  }

  return actionSceneEvent(
    actionMap,
    event,
    isFeedback ? "feedback" : "focus",
    focusState.updatedAt || now,
  );
}

function focusFeedbackEvent(event: PetActionEvent) {
  return [
    "focus-complete",
    "focus-cancel",
    "break-complete",
  ].includes(event);
}

function focusActionEvent(focusState: FocusState): PetActionEvent | null {
  if (focusState.mode === "focus") {
    switch (focusState.status) {
      case "running":
        return "focus-resume";
      case "paused":
        return "focus-pause";
      case "complete":
        return "focus-complete";
      default:
        return null;
    }
  }

  if (focusState.mode === "break") {
    switch (focusState.status) {
      case "running":
        return "break-start";
      case "paused":
        return "focus-pause";
      case "complete":
        return "break-complete";
      default:
        return null;
    }
  }

  return null;
}
