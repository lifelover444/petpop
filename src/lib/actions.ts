import { ANIMATION_BY_STATE, type PetAnimationState } from "./animations";

export type PetActionEvent =
  | "drag-left"
  | "drag-right"
  | "drag-start"
  | "drag-end"
  | "click"
  | "double-click"
  | "idle"
  | "waiting"
  | "task-running"
  | "success"
  | "error"
  | "review"
  | "codex-running"
  | "codex-waiting"
  | "codex-review"
  | "codex-success"
  | "codex-error"
  | "focus-start"
  | "focus-pause"
  | "focus-resume"
  | "focus-complete"
  | "focus-cancel"
  | "break-start"
  | "break-complete";

export type PetActionMap = Partial<Record<PetActionEvent, PetAnimationState>>;

export interface PetActionEventInfo {
  event: PetActionEvent;
  label: string;
  group: "basic" | "codex" | "focus";
}

export const ACTION_EVENTS: PetActionEventInfo[] = [
  { event: "drag-left", label: "向左拖动", group: "basic" },
  { event: "drag-right", label: "向右拖动", group: "basic" },
  { event: "click", label: "单击", group: "basic" },
  { event: "double-click", label: "双击", group: "basic" },
  { event: "idle", label: "待机", group: "basic" },
  { event: "waiting", label: "长时间未操作", group: "basic" },
  { event: "codex-running", label: "Codex 运行中", group: "codex" },
  { event: "codex-waiting", label: "Codex 等待输入", group: "codex" },
  { event: "codex-review", label: "Codex 审阅", group: "codex" },
  { event: "codex-success", label: "Codex 成功", group: "codex" },
  { event: "codex-error", label: "Codex 失败", group: "codex" },
  { event: "focus-start", label: "开始专注", group: "focus" },
  { event: "focus-pause", label: "暂停专注", group: "focus" },
  { event: "focus-resume", label: "继续专注", group: "focus" },
  { event: "focus-complete", label: "完成专注", group: "focus" },
  { event: "focus-cancel", label: "取消专注", group: "focus" },
  { event: "break-start", label: "开始休息", group: "focus" },
  { event: "break-complete", label: "完成休息", group: "focus" },
];

export const VISIBLE_ACTION_EVENTS = ACTION_EVENTS;

export const DEFAULT_ACTION_MAP: Required<PetActionMap> = {
  "drag-left": "running-left",
  "drag-right": "running-right",
  "drag-start": "waving",
  "drag-end": "idle",
  click: "waving",
  "double-click": "jumping",
  idle: "idle",
  waiting: "waiting",
  "task-running": "running",
  success: "jumping",
  error: "failed",
  review: "review",
  "codex-running": "running",
  "codex-waiting": "waiting",
  "codex-review": "review",
  "codex-success": "jumping",
  "codex-error": "failed",
  "focus-start": "running",
  "focus-pause": "waiting",
  "focus-resume": "running",
  "focus-complete": "jumping",
  "focus-cancel": "idle",
  "break-start": "waving",
  "break-complete": "jumping",
};

const ACTION_EVENT_SET = new Set<PetActionEvent>(
  Object.keys(DEFAULT_ACTION_MAP) as PetActionEvent[],
);

export function normalizeActionMap(actionMap?: PetActionMap | null) {
  const normalized: Required<PetActionMap> = { ...DEFAULT_ACTION_MAP };

  if (!actionMap) {
    return normalized;
  }

  for (const [event, state] of Object.entries(actionMap)) {
    if (!ACTION_EVENT_SET.has(event as PetActionEvent)) {
      continue;
    }

    if (!state || !ANIMATION_BY_STATE[state]) {
      continue;
    }

    normalized[event as PetActionEvent] = state;
  }

  return normalized;
}

export function resolvePetAction(
  actionMap: PetActionMap | undefined | null,
  event: PetActionEvent,
) {
  return normalizeActionMap(actionMap)[event];
}
