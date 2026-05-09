export type PetAnimationState =
  | "idle"
  | "running-right"
  | "running-left"
  | "waving"
  | "jumping"
  | "failed"
  | "waiting"
  | "running"
  | "review";

export interface AnimationRow {
  state: PetAnimationState;
  label: string;
  row: number;
  frames: number;
  durations: number[];
}

export const CELL_WIDTH = 192;
export const CELL_HEIGHT = 208;
export const ATLAS_COLUMNS = 8;
export const ATLAS_ROWS = 9;
export const ATLAS_WIDTH = CELL_WIDTH * ATLAS_COLUMNS;
export const ATLAS_HEIGHT = CELL_HEIGHT * ATLAS_ROWS;

export const ANIMATION_ROWS: AnimationRow[] = [
  {
    state: "idle",
    label: "Idle",
    row: 0,
    frames: 6,
    durations: [280, 110, 110, 140, 140, 320],
  },
  {
    state: "running-right",
    label: "Run Right",
    row: 1,
    frames: 8,
    durations: [120, 120, 120, 120, 120, 120, 120, 220],
  },
  {
    state: "running-left",
    label: "Run Left",
    row: 2,
    frames: 8,
    durations: [120, 120, 120, 120, 120, 120, 120, 220],
  },
  {
    state: "waving",
    label: "Waving",
    row: 3,
    frames: 4,
    durations: [140, 140, 140, 280],
  },
  {
    state: "jumping",
    label: "Jumping",
    row: 4,
    frames: 5,
    durations: [140, 140, 140, 140, 280],
  },
  {
    state: "failed",
    label: "Failed",
    row: 5,
    frames: 8,
    durations: [140, 140, 140, 140, 140, 140, 140, 240],
  },
  {
    state: "waiting",
    label: "Waiting",
    row: 6,
    frames: 6,
    durations: [150, 150, 150, 150, 150, 260],
  },
  {
    state: "running",
    label: "Running",
    row: 7,
    frames: 6,
    durations: [120, 120, 120, 120, 120, 220],
  },
  {
    state: "review",
    label: "Review",
    row: 8,
    frames: 6,
    durations: [150, 150, 150, 150, 150, 280],
  },
];

export const ANIMATION_BY_STATE = Object.fromEntries(
  ANIMATION_ROWS.map((row) => [row.state, row]),
) as Record<PetAnimationState, AnimationRow>;

export function framePosition(state: PetAnimationState, frame: number) {
  const row = ANIMATION_BY_STATE[state];
  const safeFrame = Math.max(0, Math.min(frame, row.frames - 1));

  return {
    x: safeFrame * CELL_WIDTH,
    y: row.row * CELL_HEIGHT,
    width: CELL_WIDTH,
    height: CELL_HEIGHT,
  };
}

export function nextFrameDelay(state: PetAnimationState, frame: number) {
  const row = ANIMATION_BY_STATE[state];
  return row.durations[frame % row.durations.length] ?? 140;
}
