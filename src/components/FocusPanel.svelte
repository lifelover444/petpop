<script lang="ts">
  import { Window } from "@tauri-apps/api/window";
  import {
    getAppSettings,
    getRuntimeState,
    isTauri,
    setFocusState,
    type AppSettings,
    type RuntimeState,
  } from "../lib/petpop";

  let runtime = $state<RuntimeState>({
    activePetId: null,
    scene: "idle",
    scale: 0.5,
    focusState: {
      mode: "idle",
      status: "idle",
      lastEvent: null,
      remainingMs: null,
      endsAt: null,
      updatedAt: Date.now(),
    },
    codexActivity: {
      status: "idle",
      message: null,
      updatedAt: 0,
    },
    codexActivityError: null,
  });
  let appSettings = $state<AppSettings>({
    focusMinutes: 25,
    breakMinutes: 5,
  });
  let clockNow = $state(Date.now());

  const remainingMs = $derived(currentFocusRemainingMs(clockNow));
  const panelTitle = $derived(
    runtime.focusState.mode === "break" ? "休息" : "专注",
  );
  const panelTime = $derived(formatPanelTime(remainingMs));
  const progressPercent = $derived(`${Math.round(focusProgress() * 100)}%`);
  const panelStatus = $derived(focusStatusLabel());
  const primaryActionLabel = $derived(
    runtime.focusState.status === "running"
      ? "暂停"
      : runtime.focusState.status === "paused"
        ? "继续"
        : "开始",
  );

  $effect(() => {
    document.documentElement.classList.add("focus-panel-runtime");
    document.body.classList.add("focus-panel-runtime");
    void refresh();
    const refreshId = window.setInterval(refresh, 1000);
    const clockId = window.setInterval(() => {
      const now = Date.now();
      clockNow = now;
      if (
        runtime.focusState.status === "running" &&
        runtime.focusState.endsAt &&
        runtime.focusState.endsAt <= now
      ) {
        void completeFocus();
      }
    }, 500);

    return () => {
      document.documentElement.classList.remove("focus-panel-runtime");
      document.body.classList.remove("focus-panel-runtime");
      window.clearInterval(refreshId);
      window.clearInterval(clockId);
    };
  });

  async function refresh() {
    const [nextRuntime, nextSettings] = await Promise.all([
      getRuntimeState(),
      getAppSettings(),
    ]);
    runtime = nextRuntime;
    appSettings = nextSettings;
  }

  async function primaryAction() {
    if (runtime.focusState.status === "running") {
      await pauseFocus();
      return;
    }

    if (runtime.focusState.status === "paused") {
      await resumeFocus();
      return;
    }

    await startFocus();
  }

  async function startFocus() {
    const durationMs = appSettings.focusMinutes * 60_000;
    runtime = await setFocusState({
      mode: "focus",
      status: "running",
      lastEvent: "focus-start",
      remainingMs: durationMs,
      endsAt: Date.now() + durationMs,
    });
  }

  async function pauseFocus() {
    runtime = await setFocusState({
      mode: runtime.focusState.mode,
      status: "paused",
      lastEvent: "focus-pause",
      remainingMs: currentFocusRemainingMs(Date.now()),
      endsAt: null,
    });
  }

  async function resumeFocus() {
    const remaining = Math.max(1000, currentFocusRemainingMs(Date.now()));
    runtime = await setFocusState({
      mode: runtime.focusState.mode,
      status: "running",
      lastEvent:
        runtime.focusState.mode === "break" ? "break-start" : "focus-resume",
      remainingMs: remaining,
      endsAt: Date.now() + remaining,
    });
  }

  async function completeFocus() {
    const isBreak = runtime.focusState.mode === "break";
    runtime = await setFocusState({
      mode: runtime.focusState.mode,
      status: "complete",
      lastEvent: isBreak ? "break-complete" : "focus-complete",
      remainingMs: 0,
      endsAt: null,
    });
  }

  async function resetFocus() {
    runtime = await setFocusState({
      mode: "idle",
      status: "idle",
      lastEvent: "focus-cancel",
      remainingMs: null,
      endsAt: null,
    });
  }

  async function openMainPanel() {
    if (!isTauri()) {
      return;
    }

    const main = await Window.getByLabel("main");
    if (main) {
      await main.show();
      await main.setFocus();
    }
    await closePanel();
  }

  async function closePanel() {
    if (isTauri()) {
      await Window.getCurrent().hide();
    }
  }

  function currentFocusRemainingMs(now: number) {
    const { focusState } = runtime;
    if (focusState.mode === "idle") {
      return appSettings.focusMinutes * 60_000;
    }
    if (focusState.status === "running" && focusState.endsAt) {
      return Math.max(0, focusState.endsAt - now);
    }
    return Math.max(0, focusState.remainingMs ?? 0);
  }

  function formatPanelTime(value: number) {
    const totalSeconds = Math.ceil(value / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes.toString().padStart(2, "0")}:${seconds
      .toString()
      .padStart(2, "0")}`;
  }

  function focusProgress() {
    if (runtime.focusState.mode === "idle") {
      return 0;
    }

    const totalMs =
      runtime.focusState.mode === "break"
        ? appSettings.breakMinutes * 60_000
        : appSettings.focusMinutes * 60_000;
    if (totalMs <= 0) {
      return 0;
    }

    return Math.max(0, Math.min(1, 1 - remainingMs / totalMs));
  }

  function focusStatusLabel() {
    if (runtime.focusState.status === "running") {
      return runtime.focusState.mode === "break" ? "休息中" : "专注中";
    }
    if (runtime.focusState.status === "paused") {
      return "已暂停";
    }
    if (runtime.focusState.status === "complete") {
      return "已完成";
    }
    return "未开始";
  }
</script>

<section
  class="focus-capsule"
  data-mode={runtime.focusState.mode}
  data-status={runtime.focusState.status}
  aria-label="专注模式"
>
  <div class="capsule-meta">
    <span>{panelTitle}</span>
    <strong>{panelTime}</strong>
    <small>{panelStatus}</small>
  </div>

  <div class="capsule-actions">
    <button
      class="capsule-primary"
      aria-label={primaryActionLabel}
      title={primaryActionLabel}
      onclick={primaryAction}
    >
      {#if runtime.focusState.status === "running"}
        <span class="pause-icon"></span>
      {:else}
        <span class="play-icon"></span>
      {/if}
      <span>{primaryActionLabel}</span>
    </button>

    <button class="reset-button" aria-label="重置计时" title="重置计时" onclick={resetFocus}>重置</button>
    <button
      class="icon-button more-icon"
      aria-label="更多"
      title="更多"
      onclick={openMainPanel}
    ></button>
  </div>

  <div class="capsule-progress" style={`--progress: ${progressPercent}`}></div>
</section>

<style>
  :global(html.focus-panel-runtime),
  :global(body.focus-panel-runtime),
  :global(body.focus-panel-runtime #app) {
    width: 100%;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    background: transparent;
  }

  .focus-capsule {
    position: relative;
    width: 396px;
    height: 116px;
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: 18px;
    padding: 18px 18px 20px 22px;
    overflow: hidden;
    color: #1b1f1d;
    background: rgba(255, 253, 248, 0.96);
    border: 1px solid rgba(49, 54, 51, 0.1);
    border-radius: 24px;
    box-shadow:
      0 14px 32px rgba(32, 37, 35, 0.15),
      inset 0 0 0 1px rgba(255, 255, 255, 0.75);
    user-select: none;
  }

  .capsule-meta {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 3px 10px;
    align-items: baseline;
    min-width: 0;
  }

  .capsule-meta span,
  .capsule-meta small {
    color: #676b64;
    letter-spacing: 0;
  }

  .capsule-meta span {
    font-size: 13px;
    font-weight: 650;
  }

  .capsule-meta small {
    grid-column: 1 / -1;
    overflow: hidden;
    font-size: 12px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .capsule-meta strong {
    grid-column: 1 / -1;
    display: block;
    overflow: hidden;
    color: #202523;
    font-size: 42px;
    font-weight: 700;
    line-height: 0.96;
    letter-spacing: 0;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .capsule-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .capsule-primary {
    min-width: 76px;
    min-height: 36px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    border: 0;
    border-radius: 999px;
    padding: 0 14px;
    color: #ffffff;
    background: #202523;
    box-shadow: 0 8px 18px rgba(32, 37, 35, 0.16);
  }

  .capsule-primary:hover {
    border-color: transparent;
    background: #111513;
    transform: none;
  }

  .icon-button {
    width: 32px;
    height: 32px;
    min-height: 32px;
    display: grid;
    place-items: center;
    border: 1px solid #d4d8df;
    border-radius: 50%;
    padding: 0;
    background: #ffffff;
    box-shadow: 0 1px 2px rgba(15, 23, 42, 0.12);
  }

  .icon-button:hover {
    transform: none;
    border-color: #9da3aa;
  }

  .more-icon {
    position: relative;
  }

  .reset-button {
    min-width: 48px;
    min-height: 32px;
    border: 1px solid #d4d8df;
    border-radius: 999px;
    padding: 0 12px;
    color: #4d554f;
    background: #ffffff;
    box-shadow: 0 1px 2px rgba(15, 23, 42, 0.08);
  }

  .reset-button:hover {
    border-color: #9da3aa;
    color: #202523;
    background: #f8faf8;
    transform: none;
  }

  .play-icon {
    width: 0;
    height: 0;
    margin-left: 2px;
    border-top: 6px solid transparent;
    border-bottom: 6px solid transparent;
    border-left: 9px solid #ffffff;
  }

  .pause-icon {
    width: 12px;
    height: 14px;
    border-right: 4px solid #ffffff;
    border-left: 4px solid #ffffff;
  }

  .more-icon::before {
    content: "";
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: #111827;
    box-shadow:
      -8px 0 0 #111827,
      8px 0 0 #111827;
  }

  .capsule-progress {
    position: absolute;
    right: 18px;
    bottom: 12px;
    left: 22px;
    height: 3px;
    overflow: hidden;
    border-radius: 999px;
    background: rgba(32, 37, 35, 0.1);
  }

  .capsule-progress::before {
    content: "";
    display: block;
    width: var(--progress);
    height: 100%;
    border-radius: inherit;
    background: #2f7d4f;
    transition: width 240ms ease;
  }

</style>
