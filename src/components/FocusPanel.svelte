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
    runtime.focusState.mode === "break" ? "休息时段" : "专注时段",
  );
  const panelTime = $derived(formatPanelTime(remainingMs));
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
    if (runtime.focusState.mode === "idle") {
      return `${appSettings.focusMinutes}分钟`;
    }

    const totalSeconds = Math.ceil(value / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes.toString().padStart(2, "0")}:${seconds
      .toString()
      .padStart(2, "0")}`;
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

<section class="focus-popover" aria-label="专注模式">
  <header>
    <button class="icon-button outline-icon" aria-label="打开主面板" onclick={openMainPanel}></button>
    <h1>{panelTitle}</h1>
    <button class="icon-button close-icon" aria-label="关闭" onclick={closePanel}></button>
  </header>

  <div class="timer-ring">
    <span>{panelTime}</span>
  </div>

  <div class="quick-actions">
    <button
      class="primary-button"
      aria-label={primaryActionLabel}
      title={primaryActionLabel}
      onclick={primaryAction}
    >
      {#if runtime.focusState.status === "running"}
        <span class="pause-icon"></span>
      {:else}
        <span class="play-icon"></span>
      {/if}
    </button>
    <button class="icon-button reset-icon" aria-label="重置" title="重置" onclick={resetFocus}></button>
    <button class="icon-button more-icon" aria-label="更多" title="更多" onclick={openMainPanel}></button>
  </div>

  <p>{panelStatus}</p>
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

  .focus-popover {
    width: 390px;
    height: 318px;
    display: grid;
    grid-template-rows: auto 1fr auto auto;
    justify-items: center;
    gap: 16px;
    border: 1px solid #b9bec7;
    border-radius: 14px;
    padding: 20px 24px 22px;
    color: #1b1f24;
    background: #f2f5fb;
    box-shadow: 0 18px 40px rgba(15, 23, 42, 0.18);
  }

  header {
    width: 100%;
    display: grid;
    grid-template-columns: 44px 1fr 44px;
    align-items: center;
  }

  h1 {
    margin: 0;
    text-align: center;
    font-size: 24px;
    font-weight: 500;
    letter-spacing: 0;
  }

  .timer-ring {
    width: 124px;
    height: 124px;
    display: grid;
    place-items: center;
    align-self: center;
    border: 1px solid #e0e4ea;
    border-radius: 50%;
    background:
      radial-gradient(circle, #ffffff 0 62%, transparent 63%),
      repeating-conic-gradient(#e8ecf2 0deg 3deg, transparent 3deg 15deg);
  }

  .timer-ring span {
    min-width: 82px;
    text-align: center;
    font-size: 24px;
    letter-spacing: 0;
  }

  .quick-actions {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .icon-button,
  .primary-button {
    width: 42px;
    height: 42px;
    min-height: 42px;
    display: grid;
    place-items: center;
    border: 1px solid #d4d8df;
    border-radius: 50%;
    padding: 0;
    background: #ffffff;
    box-shadow: 0 1px 2px rgba(15, 23, 42, 0.15);
  }

  .primary-button {
    border-color: #0578d4;
    background: #0578d4;
    box-shadow: none;
  }

  .icon-button:hover,
  .primary-button:hover {
    transform: none;
    border-color: #0578d4;
  }

  .outline-icon,
  .close-icon,
  .reset-icon,
  .more-icon {
    position: relative;
  }

  .outline-icon::before,
  .outline-icon::after {
    content: "";
    position: absolute;
    width: 16px;
    height: 16px;
    border: 2px solid #1b1f24;
  }

  .outline-icon::before {
    top: 7px;
    left: 7px;
    border-right: 0;
    border-bottom: 0;
  }

  .outline-icon::after {
    right: 7px;
    bottom: 7px;
    border-left: 0;
    border-top: 0;
  }

  .close-icon::before,
  .close-icon::after {
    content: "";
    position: absolute;
    width: 25px;
    height: 2px;
    border-radius: 999px;
    background: #111827;
  }

  .close-icon::before {
    transform: rotate(45deg);
  }

  .close-icon::after {
    transform: rotate(-45deg);
  }

  .play-icon {
    width: 0;
    height: 0;
    margin-left: 3px;
    border-top: 10px solid transparent;
    border-bottom: 10px solid transparent;
    border-left: 15px solid #ffffff;
  }

  .pause-icon {
    width: 16px;
    height: 18px;
    border-right: 5px solid #ffffff;
    border-left: 5px solid #ffffff;
  }

  .reset-icon::before {
    content: "";
    width: 21px;
    height: 21px;
    border: 2px solid #111827;
    border-right-color: transparent;
    border-radius: 50%;
  }

  .reset-icon::after {
    content: "";
    position: absolute;
    top: 9px;
    left: 10px;
    width: 8px;
    height: 8px;
    border-top: 2px solid #111827;
    border-left: 2px solid #111827;
    transform: rotate(-32deg);
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

  p {
    margin: -6px 0 0;
    color: #5c626b;
    font-size: 24px;
    letter-spacing: 0;
  }
</style>
