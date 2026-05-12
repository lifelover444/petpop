import type { PetAnimationState } from "./animations";

export type SceneSource =
  | "interaction"
  | "feedback"
  | "movement"
  | "codex"
  | "focus"
  | "system"
  | "idle";

export interface SceneEvent {
  source: SceneSource;
  state: PetAnimationState;
  timestamp: number;
  minDurationMs?: number;
}

export interface ScheduledScene {
  state: PetAnimationState;
  lockedUntil: number;
  source: SceneSource;
  timestamp?: number;
}

const PRIORITY: Record<SceneSource, number> = {
  interaction: 6,
  movement: 6,
  feedback: 5,
  codex: 4,
  focus: 3,
  system: 3,
  idle: 1,
};

const DEFAULT_LOCK_MS: Record<SceneSource, number> = {
  interaction: 900,
  feedback: 1200,
  system: 1200,
  movement: 0,
  codex: 0,
  focus: 0,
  idle: 0,
};

export function selectScene(
  current: ScheduledScene,
  events: SceneEvent[],
  now: number,
): ScheduledScene {
  if (now < current.lockedUntil) {
    const strongerEvent = events
      .filter(
        (event) =>
          PRIORITY[event.source] > PRIORITY[current.source] ||
          (event.source === current.source &&
            event.timestamp > (current.timestamp ?? -Infinity)),
      )
      .sort(compareSceneEvents)[0];

    if (!strongerEvent) {
      return current;
    }

    return lockScene(strongerEvent, now);
  }

  const next = events.sort(compareSceneEvents)[0];
  return next ? lockScene(next, now) : current;
}

export function idleScene(now: number, inactiveMs: number): SceneEvent {
  return {
    source: "idle",
    state: inactiveMs >= 60_000 ? "waiting" : "idle",
    timestamp: now,
  };
}

export function initialScene(now = Date.now()): ScheduledScene {
  return {
    state: "idle",
    source: "idle",
    lockedUntil: now,
    timestamp: now,
  };
}

function compareSceneEvents(a: SceneEvent, b: SceneEvent) {
  const priorityDelta = PRIORITY[b.source] - PRIORITY[a.source];
  if (priorityDelta !== 0) {
    return priorityDelta;
  }

  return b.timestamp - a.timestamp;
}

function lockScene(event: SceneEvent, now: number): ScheduledScene {
  return {
    state: event.state,
    source: event.source,
    timestamp: event.timestamp,
    lockedUntil:
      now + (event.minDurationMs ?? DEFAULT_LOCK_MS[event.source] ?? 0),
  };
}
