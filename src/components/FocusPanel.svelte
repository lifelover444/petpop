<script lang="ts">
  import { LogicalSize, PhysicalSize } from "@tauri-apps/api/dpi";
  import { Window } from "@tauri-apps/api/window";
  import {
    getAppSettings,
    getRuntimeState,
    isTauri,
    setAppSettings,
    setFocusState,
    showMainWindow,
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
  let capsuleOpen = $state(false);
  const FOCUS_LAUNCHER_WIDTH = 132;
  const FOCUS_LAUNCHER_HEIGHT = 46;
  const FOCUS_PANEL_WIDTH = 800;
  const FOCUS_PANEL_HEIGHT = 154;

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
        : "开始专注",
  );

  $effect(() => {
    document.documentElement.classList.add("focus-panel-runtime");
    document.body.classList.add("focus-panel-runtime");
    void refresh();
    const unlisteners: Array<() => void> = [];
    if (isTauri()) {
      const currentWindow = Window.getCurrent();
      void syncCapsuleModeToWindowSize(currentWindow);
      void currentWindow
        .listen("focus-panel:launcher", () => {
          capsuleOpen = false;
        })
        .then((unlisten) => {
          unlisteners.push(unlisten);
        });
      void currentWindow
        .onResized(({ payload: size }) => {
          void syncCapsuleModeToWindowSize(currentWindow, size);
        })
        .then((unlisten) => {
          unlisteners.push(unlisten);
        });
      void currentWindow
        .onFocusChanged(({ payload: focused }) => {
          if (!focused) {
            void hideLauncherOnBlur();
            return;
          }

          void syncCapsuleModeToWindowSize(currentWindow);
        })
        .then((unlisten) => {
          unlisteners.push(unlisten);
        });
    }
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
      unlisteners.forEach((unlisten) => unlisten());
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

  async function startBreak() {
    const durationMs = appSettings.breakMinutes * 60_000;
    runtime = await setFocusState({
      mode: "break",
      status: "running",
      lastEvent: "break-start",
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

  async function updateFocusMinutes(value: number) {
    appSettings = await setAppSettings({
      ...appSettings,
      focusMinutes: Math.max(1, Math.min(180, Math.round(value || 1))),
    });
  }

  async function updateBreakMinutes(value: number) {
    appSettings = await setAppSettings({
      ...appSettings,
      breakMinutes: Math.max(1, Math.min(60, Math.round(value || 1))),
    });
  }

  async function syncCapsuleModeToWindowSize(
    currentWindow = Window.getCurrent(),
    size?: PhysicalSize,
  ) {
    const currentSize = size ?? (await currentWindow.innerSize());
    const scaleFactor = await currentWindow.scaleFactor().catch(() => 1);
    if (
      currentSize.width <= FOCUS_LAUNCHER_WIDTH * scaleFactor + 1 &&
      currentSize.height <= FOCUS_LAUNCHER_HEIGHT * scaleFactor + 1
    ) {
      capsuleOpen = false;
    }
  }

  async function hideLauncherOnBlur() {
    if (!capsuleOpen) {
      await Window.getCurrent().hide();
    }
  }

  function startDraggingFocusPanel(event: PointerEvent) {
    if (event.button !== 0 || !isTauri() || isInteractiveTarget(event.target)) {
      return;
    }

    event.preventDefault();
    void Window.getCurrent().startDragging();
  }

  function isInteractiveTarget(target: EventTarget | null) {
    return (
      target instanceof Element &&
      Boolean(target.closest("button, input, select, textarea, a, label"))
    );
  }

  async function openCapsule() {
    capsuleOpen = true;
    if (isTauri()) {
      await Window.getCurrent().setSize(
        new LogicalSize(FOCUS_PANEL_WIDTH, FOCUS_PANEL_HEIGHT),
      );
    }
  }

  async function openMainPanel() {
    if (!isTauri()) {
      return;
    }

    await showMainWindow();
    await closePanel();
  }

  async function closePanel() {
    capsuleOpen = false;
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
      return 1;
    }

    const totalMs =
      runtime.focusState.mode === "break"
        ? appSettings.breakMinutes * 60_000
        : appSettings.focusMinutes * 60_000;
    if (totalMs <= 0) {
      return 0;
    }

    return Math.max(0, Math.min(1, remainingMs / totalMs));
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

{#if !capsuleOpen}
  <section class="focus-launcher-surface" aria-label="专注模式入口">
    <button class="focus-launcher" type="button" onclick={openCapsule}>打开专注模式</button>
  </section>
{:else}
  <section
    class="focus-capsule"
    data-mode={runtime.focusState.mode}
    data-status={runtime.focusState.status}
    aria-label="专注模式"
    onpointerdown={startDraggingFocusPanel}
  >
    <button
      class="close-button"
      aria-label="关闭气泡"
      title="关闭气泡"
      onclick={closePanel}
    ></button>

    <div class="capsule-meta">
      <span>{panelTitle}</span>
      <strong>{panelTime}</strong>
      <small>{panelStatus}</small>
    </div>

    <div class="capsule-settings">
      <label class="duration-field">
        <span>专注</span>
        <input
          class="duration-stepper"
          aria-label="专注时长"
          type="number"
          min="1"
          max="180"
          value={appSettings.focusMinutes}
          onchange={(event) =>
            updateFocusMinutes(Number((event.target as HTMLInputElement).value))}
        />
        <small>分钟</small>
      </label>

      <label class="duration-field">
        <span>休息</span>
        <input
          class="duration-stepper"
          aria-label="休息时长"
          type="number"
          min="1"
          max="60"
          value={appSettings.breakMinutes}
          onchange={(event) =>
            updateBreakMinutes(Number((event.target as HTMLInputElement).value))}
        />
        <small>分钟</small>
      </label>
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

      {#if runtime.focusState.status !== "running" && runtime.focusState.status !== "paused"}
        <button class="break-button" type="button" onclick={startBreak}>开始休息</button>
      {/if}
      <button class="reset-button" aria-label="重置计时" title="重置计时" onclick={resetFocus}>重置</button>
      <button
        class="icon-button more-icon"
        aria-label="打开桌面端"
        title="打开桌面端"
        onclick={openMainPanel}
      ></button>
    </div>

    <div class="capsule-progress" style={`--progress: ${progressPercent}`}></div>
  </section>
{/if}

<style>
  :global(html.focus-panel-runtime),
  :global(body.focus-panel-runtime),
  :global(body.focus-panel-runtime #app) {
    width: 100vw;
    height: 100vh;
    min-width: 0;
    min-height: 0;
    margin: 0;
    overflow: hidden;
    background: transparent;
  }

  .focus-launcher-surface,
  .focus-capsule {
    position: relative;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
  }

  .focus-launcher-surface {
    display: grid;
    place-items: start;
    background: transparent;
  }

  .focus-launcher,
  .focus-capsule {
    position: relative;
    color: #1b1f1d;
    background: #fffdf8;
    border: 0;
    border-radius: 24px;
    box-shadow: none;
    user-select: none;
  }

  .focus-launcher::before,
  .focus-capsule::before {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: inherit;
    pointer-events: none;
    box-shadow: inset 0 0 0 1px rgba(49, 54, 51, 0.12);
  }

  .focus-launcher {
    width: 132px;
    height: 46px;
    min-height: 46px;
    display: grid;
    place-items: center;
    padding: 0 16px;
    color: #ffffff;
    font-size: 14px;
    font-weight: 650;
    background: #202523;
  }

  .focus-launcher:hover {
    background: #111513;
    transform: none;
  }

  .focus-capsule {
    display: grid;
    grid-template-columns: 176px minmax(248px, 1fr) auto;
    align-items: center;
    gap: 12px;
    padding: 18px 42px 20px 22px;
  }

  .close-button {
    position: absolute;
    top: 8px;
    right: 9px;
    z-index: 1;
    width: 24px;
    height: 24px;
    min-height: 24px;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: 50%;
    padding: 0;
    background: transparent;
  }

  .close-button::before,
  .close-button::after {
    content: "";
    position: absolute;
    width: 12px;
    height: 1.5px;
    border-radius: 999px;
    background: #59615a;
  }

  .close-button::before {
    transform: rotate(45deg);
  }

  .close-button::after {
    transform: rotate(-45deg);
  }

  .close-button:hover {
    background: rgba(32, 37, 35, 0.08);
    transform: none;
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
    font-size: 12px;
    white-space: nowrap;
  }

  .capsule-meta strong {
    grid-column: 1 / -1;
    display: block;
    color: #202523;
    font-size: 42px;
    font-weight: 700;
    line-height: 0.96;
    letter-spacing: 0;
    white-space: nowrap;
  }

  .capsule-settings {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
    color: #676b64;
    font-size: 12px;
    white-space: nowrap;
  }

  .duration-field {
    display: grid;
    grid-template-columns: auto 54px auto;
    align-items: center;
    gap: 6px;
  }

  .duration-field span,
  .duration-field small {
    color: #676b64;
  }

  .duration-stepper {
    width: 54px;
    min-height: 30px;
    border-color: #d4d8df;
    border-radius: 999px;
    padding: 0 6px;
    text-align: center;
    background: #ffffff;
  }

  .capsule-actions {
    display: flex;
    align-items: center;
    flex-wrap: nowrap;
    gap: 8px;
  }

  .capsule-primary {
    min-width: 92px;
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
    white-space: nowrap;
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
    white-space: nowrap;
  }

  .break-button {
    min-width: 76px;
    min-height: 32px;
    border: 1px solid #d4d8df;
    border-radius: 999px;
    padding: 0 12px;
    color: #265f3d;
    background: #f4faf5;
    box-shadow: 0 1px 2px rgba(15, 23, 42, 0.08);
    white-space: nowrap;
  }

  .break-button:hover {
    border-color: #9ab9a4;
    color: #1f4e33;
    background: #edf7ef;
    transform: none;
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
