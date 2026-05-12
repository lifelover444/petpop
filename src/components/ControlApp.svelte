<script lang="ts">
  import SpritePet from "./SpritePet.svelte";
  import {
    ACTION_EVENTS,
    normalizeActionMap,
    resolvePetAction,
    type PetActionEvent,
    type PetActionMap,
  } from "../lib/actions";
  import {
    ANIMATION_ROWS,
    type PetAnimationState,
  } from "../lib/animations";
  import {
    chooseImportPath,
    getAppSettings,
    getPetSpriteUrl,
    getRuntimeState,
    importPetFromPath,
    importPetdex,
    listPets,
    removePet,
    scanCodexPets,
    setActivePet,
    setAppSettings,
    setFocusState,
    setPetActionMap,
    setScale,
    setScene,
    sourceKindLabel,
    type AppSettings,
    type FocusMode,
    type PetInfo,
    type RuntimeState,
  } from "../lib/petpop";

  let pets = $state<PetInfo[]>([]);
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
  let importPath = $state("");
  let petdexInput = $state("");
  let status = $state("就绪");
  let busy = $state(false);
  let lastInteraction = Date.now();
  let clockNow = $state(Date.now());
  let activeSpriteUrl = $state("");
  let actionMapDraft = $state<PetActionMap>({});
  let appSettings = $state<AppSettings>({
    focusMinutes: 25,
    breakMinutes: 5,
  });
  let scenePlaybackKey = $state(0);

  const activePet = $derived(
    pets.find((pet) => pet.id === runtime.activePetId) ?? pets[0],
  );
  const activeActionMap = $derived(normalizeActionMap(activePet?.actionMap));
  const actionEventGroups = $derived([
    {
      title: "基础交互",
      events: ACTION_EVENTS.filter((item) => item.group === "basic"),
    },
    {
      title: "Codex",
      events: ACTION_EVENTS.filter((item) => item.group === "codex"),
    },
    {
      title: "专注模式",
      events: ACTION_EVENTS.filter((item) => item.group === "focus"),
    },
  ]);
  const focusRemainingMs = $derived(currentFocusRemainingMs(clockNow));
  const focusLabel = $derived(formatDuration(focusRemainingMs));
  const focusStatusText = $derived(focusStatusLabel());
  const focusModeText = $derived(
    runtime.focusState.mode === "break" ? "休息" : "专注",
  );
  const codexStatusLabel = $derived(codexLabel(runtime.codexActivity.status));

  async function refresh() {
    const [nextPets, nextRuntime] = await Promise.all([
      listPets(),
      getRuntimeState(),
    ]);
    pets = nextPets;
    runtime = nextRuntime;

    if (!nextRuntime.activePetId && nextPets[0]) {
      runtime = await setActivePet(nextPets[0].id);
    }
  }

  async function loadSettings() {
    appSettings = await getAppSettings();
  }

  async function runAction<T>(message: string, action: () => Promise<T>) {
    busy = true;
    status = message;

    try {
      await action();
      status = "已完成";
      await refresh();
    } catch (error) {
      status = error instanceof Error ? error.message : String(error);
      await refresh();
    } finally {
      busy = false;
    }
  }

  async function selectPet(id: string) {
    const selected = pets.find((pet) => pet.id === id);
    runtime = await setActivePet(id);
    await triggerAction("click", selected?.actionMap);
    await refresh();
  }

  async function importPathNow() {
    if (!importPath.trim()) {
      status = "请选择宠物包或文件夹路径";
      return;
    }

    await runAction("正在导入", async () => {
      const pet = await importPetFromPath(importPath.trim());
      runtime = await setActivePet(pet.id);
      importPath = "";
    });
  }

  async function pickPath(kind: "file" | "folder") {
    const path = await chooseImportPath(kind);
    if (path) {
      importPath = path;
    }
  }

  async function importPetdexNow() {
    if (!petdexInput.trim()) {
      status = "请输入 PetDex ID 或链接";
      return;
    }

    await runAction("正在从 PetDex 导入", async () => {
      const pet = await importPetdex(petdexInput.trim());
      runtime = await setActivePet(pet.id);
      petdexInput = "";
    });
  }

  async function scanCodexNow() {
    await runAction("正在扫描 Codex 宠物", async () => {
      const imported = await scanCodexPets();
      if (imported[0]) {
        runtime = await setActivePet(imported[0].id);
      }
    });
  }

  async function removeActivePet() {
    if (!activePet) {
      return;
    }

    const confirmed = window.confirm(
      `移除“${activePet.displayName}”？这只会删除 PetPop 的导入副本，不会修改原始宠物文件。`,
    );
    if (!confirmed) {
      return;
    }

    await runAction("正在移除宠物", async () => {
      runtime = await removePet(activePet.id);
      activeSpriteUrl = "";
    });
  }

  async function triggerAction(event: PetActionEvent, actionMap = activePet?.actionMap) {
    lastInteraction = Date.now();
    scenePlaybackKey += 1;
    runtime = await setScene(resolvePetAction(actionMap, event));
  }

  async function triggerScene(state: PetAnimationState) {
    const now = Date.now();
    lastInteraction = now;
    scenePlaybackKey += 1;
    runtime = await setScene(state);
  }

  async function updateScale(value: number) {
    runtime = await setScale(value);
  }

  async function saveFocusSettings() {
    appSettings = await setAppSettings(appSettings);
    status = "专注设置已保存";
  }

  async function updateFocusMinutes(value: number) {
    appSettings = {
      ...appSettings,
      focusMinutes: Math.max(1, Math.min(180, Math.round(value || 1))),
    };
    await saveFocusSettings();
  }

  async function updateBreakMinutes(value: number) {
    appSettings = {
      ...appSettings,
      breakMinutes: Math.max(1, Math.min(60, Math.round(value || 1))),
    };
    await saveFocusSettings();
  }

  async function startFocus() {
    await startTimedState("focus", appSettings.focusMinutes, "focus-start");
    status = "专注中";
  }

  async function startBreak() {
    await startTimedState("break", appSettings.breakMinutes, "break-start");
    status = "休息中";
  }

  async function startTimedState(
    mode: FocusMode,
    minutes: number,
    event: PetActionEvent,
  ) {
    const durationMs = minutes * 60_000;
    runtime = await setFocusState({
      mode,
      status: "running",
      lastEvent: event,
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
    status = runtime.focusState.mode === "break" ? "休息已暂停" : "专注已暂停";
  }

  async function resumeFocus() {
    const remainingMs = Math.max(1000, currentFocusRemainingMs(Date.now()));
    const event =
      runtime.focusState.mode === "break" ? "break-start" : "focus-resume";
    runtime = await setFocusState({
      mode: runtime.focusState.mode,
      status: "running",
      lastEvent: event,
      remainingMs,
      endsAt: Date.now() + remainingMs,
    });
    status = runtime.focusState.mode === "break" ? "休息中" : "专注中";
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
    status = isBreak ? "休息完成" : "专注完成";
  }

  async function cancelFocus() {
    runtime = await setFocusState({
      mode: "idle",
      status: "idle",
      lastEvent: "focus-cancel",
      remainingMs: null,
      endsAt: null,
    });
    status = "专注已结束";
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

  function formatDuration(value: number) {
    const totalSeconds = Math.ceil(value / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes.toString().padStart(2, "0")}:${seconds
      .toString()
      .padStart(2, "0")}`;
  }

  function codexLabel(status: RuntimeState["codexActivity"]["status"]) {
    switch (status) {
      case "running":
        return "运行中";
      case "waiting":
        return "等待输入";
      case "review":
        return "审阅";
      case "success":
        return "成功";
      case "error":
        return "失败";
      default:
        return "空闲";
    }
  }

  async function updateMappedAction(
    event: PetActionEvent,
    state: PetAnimationState,
  ) {
    const nextMap = { ...actionMapDraft, [event]: state };
    actionMapDraft = nextMap;
    await persistActionMap(nextMap, false);
  }

  async function saveActionMap() {
    await persistActionMap(actionMapDraft, true);
  }

  function focusStatusLabel() {
    switch (runtime.focusState.status) {
      case "running":
        return "进行中";
      case "paused":
        return "已暂停";
      case "complete":
        return "已完成";
      default:
        return "未开始";
    }
  }

  async function resetActionMap() {
    const nextMap = normalizeActionMap();
    actionMapDraft = nextMap;
    await persistActionMap(nextMap, true);
  }

  async function persistActionMap(actionMap: PetActionMap, showBusy: boolean) {
    if (!activePet) {
      return;
    }

    if (showBusy) {
      busy = true;
      status = "正在保存动作映射";
    }

    try {
      const pet = await setPetActionMap(activePet.id, actionMap);
      pets = pets.map((item) => (item.id === pet.id ? pet : item));
      actionMapDraft = normalizeActionMap(pet.actionMap);
      status = showBusy ? "动作映射已保存" : "已自动保存动作映射";
    } catch (error) {
      status = error instanceof Error ? error.message : String(error);
    } finally {
      if (showBusy) {
        busy = false;
      }
    }
  }

  $effect(() => {
    loadSettings();
    refresh();
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
      window.clearInterval(refreshId);
      window.clearInterval(clockId);
    };
  });

  $effect(() => {
    const pet = activePet;
    let cancelled = false;
    activeSpriteUrl = "";

    if (!pet) {
      return;
    }

    getPetSpriteUrl(pet)
      .then((url) => {
        if (!cancelled) {
          activeSpriteUrl = url;
        }
      })
      .catch((error) => {
        if (!cancelled) {
          status = error instanceof Error ? error.message : String(error);
        }
      });

    return () => {
      cancelled = true;
    };
  });

  $effect(() => {
    actionMapDraft = normalizeActionMap(activePet?.actionMap);
    JSON.stringify(activePet?.actionMap ?? {});
  });
</script>

<main class="app-shell">
  <section class="library">
    <div class="brand">
      <span class="mark"></span>
      <div>
        <h1>PetPop</h1>
        <p>通用桌宠运行时</p>
      </div>
    </div>

    <div class="pet-list" aria-label="宠物库">
      {#if pets.length === 0}
        <div class="empty-list">还没有导入宠物</div>
      {:else}
        {#each pets as pet}
          <button
            class:active={pet.id === activePet?.id}
            onclick={() => selectPet(pet.id)}
          >
            <span>{pet.displayName}</span>
            <small>{sourceKindLabel(pet.sourceKind)}</small>
          </button>
        {/each}
      {/if}
    </div>
  </section>

  <section
    class="workspace"
    role="application"
    onmousemove={() => (lastInteraction = Date.now())}
  >
    <header class="topbar">
      <div>
        <h2>{activePet?.displayName ?? "未选择宠物"}</h2>
        <p>{activePet?.description ?? "请在桌面应用中导入 Codex 兼容宠物包。"}</p>
      </div>
      <div class="topbar-actions">
        <button
          class="danger-button"
          disabled={busy || !activePet}
          onclick={removeActivePet}
        >
          移除宠物
        </button>
        <div class="status" class:busy>{status}</div>
      </div>
    </header>

    <div class="stage">
      {#if activePet && activeSpriteUrl}
        <SpritePet
          imageUrl={activeSpriteUrl}
          state={runtime.scene}
          scale={runtime.scale}
          playbackKey={scenePlaybackKey}
        />
      {:else if activePet}
        <div class="stage-message">正在加载精灵图</div>
      {:else}
        <div class="stage-message">还没有导入宠物</div>
      {/if}
    </div>

    <div class="controls">
      <div class="scene-grid">
        {#each ANIMATION_ROWS as row}
        <button
          class:active={runtime.scene === row.state}
          onclick={() => triggerScene(row.state)}
        >
          {row.label}
        </button>
      {/each}
      </div>

      <label class="scale-control">
        <span>缩放</span>
        <input
          type="range"
          min="0.1"
          max="1"
          step="0.05"
          value={runtime.scale}
          oninput={(event) =>
            updateScale(Number((event.target as HTMLInputElement).value))}
        />
        <strong>{runtime.scale.toFixed(2)}x</strong>
      </label>

      <div class="runtime-status">
        <span>Codex</span>
        <strong>{codexStatusLabel}</strong>
        {#if runtime.codexActivityError}
          <small>{runtime.codexActivityError}</small>
        {:else if runtime.codexActivity.message}
          <small>{runtime.codexActivity.message}</small>
        {/if}
      </div>
    </div>

    <div class="focus-panel">
      <div class="focus-summary">
        <div>
          <span>专注模式</span>
          <small>{focusModeText} · {focusStatusText}</small>
        </div>
        <strong>{focusLabel}</strong>
      </div>

      <div class="focus-settings" aria-label="专注时长设置">
        <label class="duration-field">
          <span>专注</span>
          <input
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

      <div class="focus-actions">
        {#if runtime.focusState.status === "running"}
          <button onclick={pauseFocus}>暂停</button>
          <button onclick={completeFocus}>完成</button>
          <button onclick={cancelFocus}>结束</button>
        {:else if runtime.focusState.status === "paused"}
          <button onclick={resumeFocus}>继续</button>
          <button onclick={completeFocus}>完成</button>
          <button onclick={cancelFocus}>结束</button>
        {:else}
          <button onclick={startFocus}>开始专注</button>
          <button onclick={startBreak}>开始休息</button>
        {/if}
      </div>
    </div>

    <div class="action-map">
      <div class="action-map-header">
        <h2>动作映射</h2>
        <div class="action-map-buttons">
          <button disabled={busy || !activePet} onclick={resetActionMap}>
            恢复 Codex 默认
          </button>
          <button disabled={busy || !activePet} onclick={saveActionMap}>保存</button>
        </div>
      </div>
      <div class="action-map-sections">
        {#each actionEventGroups as group}
          <section>
            <h3>{group.title}</h3>
            <div class="action-map-grid">
              {#each group.events as item}
                <label>
                  <span>{item.label}</span>
                  <select
                    disabled={!activePet}
                    value={actionMapDraft[item.event] ?? activeActionMap[item.event]}
                    onchange={(event) =>
                      updateMappedAction(
                        item.event,
                        (event.target as HTMLSelectElement).value as PetAnimationState,
                      )}
                  >
                    {#each ANIMATION_ROWS as row}
                      <option value={row.state}>{row.label}</option>
                    {/each}
                  </select>
                </label>
              {/each}
            </div>
          </section>
        {/each}
      </div>
    </div>
  </section>

  <aside class="importer">
    <h2>导入</h2>

    <label>
      <span>宠物包路径</span>
      <div class="input-row">
        <input bind:value={importPath} placeholder="zip 文件或宠物文件夹" />
        <button onclick={() => pickPath("file")}>选择 Zip</button>
        <button onclick={() => pickPath("folder")}>文件夹</button>
      </div>
    </label>
    <button disabled={busy} onclick={importPathNow}>导入本地宠物</button>

    <label>
      <span>PetDex</span>
      <input bind:value={petdexInput} placeholder="boba 或 PetDex 链接" />
    </label>
    <button disabled={busy} onclick={importPetdexNow}>导入 PetDex 宠物</button>

    <button disabled={busy} onclick={scanCodexNow}>扫描 Codex 宠物</button>

    <div class="source-note">
      PetDex 宠物来自用户提交。PetPop 只保存你主动导入的宠物。
    </div>
  </aside>
</main>
