<script lang="ts">
  import { untrack } from "svelte";
  import {
    LogicalPosition,
    LogicalSize,
    PhysicalPosition,
  } from "@tauri-apps/api/dpi";
  import {
    availableMonitors,
    cursorPosition,
    getCurrentWindow,
    Window,
    type Monitor,
  } from "@tauri-apps/api/window";
  import SpritePet from "./SpritePet.svelte";
  import {
    resolvePetAction,
    type PetActionEvent,
  } from "../lib/actions";
  import { repeatedAnimationDurationMs } from "../lib/animations";
  import {
    DOUBLE_CLICK_MS,
    dragEventFromDelta,
    hasDragMovement,
    nextClickAction,
    type DragActionEvent,
  } from "../lib/petInteraction";
  import { actionSceneEvent, runtimeSceneEvents } from "../lib/runtimeScene";
  import {
    initialScene,
    selectScene,
    type ScheduledScene,
    type SceneEvent,
    type SceneSource,
  } from "../lib/sceneEngine";
  import type { PetInfo, RuntimeState } from "../lib/petpop";
  import {
    getRuntimeState,
    getPetSpriteUrl,
    getPetWindowPosition,
    isLeftMouseButtonPressed,
    isTauri,
    listPets,
    setFocusState,
    setPetWindowPosition,
    setScene,
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
  let activeSpriteUrl = $state("");
  let pressedPointerId: number | null = null;
  let pressStartX = 0;
  let pressStartY = 0;
  let clickTimer: number | undefined;
  let dragTracker: number | undefined;
  let dragButtonPoller: number | undefined;
  let dragStartVersion = 0;
  let dragCursorSampleVersion = 0;
  let lastDragCursorX: number | null = null;
  let lastDragEvent: DragActionEvent | null = null;
  let lastPointerScreenX: number | null = null;
  let lastPointerDirectionAt = 0;
  let dragMoved = false;
  let dragging = false;
  let lastInteraction = Date.now();
  let scheduledScene: ScheduledScene = initialScene();
  let scenePlaybackKey = $state(0);
  let sceneReconcileTimer: number | undefined;
  let sceneUpdateVersion = 0;
  let pendingSceneUpdates = 0;
  let sceneUpdateQueue = Promise.resolve();
  const DRAG_POINTER_DIRECTION_GRACE_MS = 120;
  const PET_WINDOW_WIDTH = 192;
  const PET_WINDOW_HEIGHT = 208;
  const FOCUS_LAUNCHER_WIDTH = 132;
  const FOCUS_LAUNCHER_HEIGHT = 46;
  const FOCUS_PANEL_WIDTH = 520;
  const FOCUS_PANEL_HEIGHT = 132;

  const activePet = $derived(
    pets.find((pet) => pet.id === runtime.activePetId) ?? pets[0],
  );
  const activePetSignature = $derived(
    activePet
      ? `${activePet.id}\n${activePet.spritesheetPath}\n${JSON.stringify(activePet.actionMap ?? {})}`
      : "",
  );

  async function refreshPets() {
    pets = await listPets();
  }

  async function refreshRuntime() {
    const nextRuntime = await getRuntimeState();
    runtime =
      pendingSceneUpdates > 0
        ? { ...nextRuntime, scene: runtime.scene }
        : nextRuntime;
    await reconcileScene();
  }

  async function restorePetWindowPosition() {
    const savedPosition = await getPetWindowPosition();
    const safePosition = await visiblePetWindowPosition(
      savedPosition.x,
      savedPosition.y,
    );

    await getCurrentWindow().setPosition(
      new PhysicalPosition(safePosition.x, safePosition.y),
    );

    if (safePosition.x !== savedPosition.x || safePosition.y !== savedPosition.y) {
      await setPetWindowPosition(safePosition);
    }
  }

  async function visiblePetWindowPosition(x: number, y: number) {
    return visibleWindowPosition(x, y, PET_WINDOW_WIDTH, PET_WINDOW_HEIGHT);
  }

  async function visibleWindowPosition(
    x: number,
    y: number,
    width: number,
    height: number,
  ) {
    const monitors = await availableMonitors();
    const monitor =
      monitors.find((item) => intersectsMonitor(x, y, item.workArea)) ??
      monitors[0];

    if (!monitor) {
      return { x, y };
    }

    const minX = monitor.workArea.position.x;
    const minY = monitor.workArea.position.y;
    const maxX = monitor.workArea.position.x + monitor.workArea.size.width - width;
    const maxY = monitor.workArea.position.y + monitor.workArea.size.height - height;

    return {
      x: clamp(Math.round(x), minX, Math.max(minX, maxX)),
      y: clamp(Math.round(y), minY, Math.max(minY, maxY)),
    };
  }

  function intersectsMonitor(
    x: number,
    y: number,
    workArea: Monitor["workArea"],
  ) {
    return (
      x + PET_WINDOW_WIDTH > workArea.position.x &&
      x < workArea.position.x + workArea.size.width &&
      y + PET_WINDOW_HEIGHT > workArea.position.y &&
      y < workArea.position.y + workArea.size.height
    );
  }

  function clamp(value: number, min: number, max: number) {
    return Math.min(Math.max(value, min), max);
  }

  $effect(() => {
    document.documentElement.classList.add("pet-runtime");
    document.body.classList.add("pet-runtime");
    if (isTauri()) {
      void restorePetWindowPosition();
    }
    refreshPets();
    refreshRuntime();
    const runtimeId = window.setInterval(refreshRuntime, 350);
    const petsId = window.setInterval(refreshPets, 1000);
    const idleId = window.setInterval(applyIdleScene, 1000);

    return () => {
      document.documentElement.classList.remove("pet-runtime");
      document.body.classList.remove("pet-runtime");
      window.clearInterval(runtimeId);
      window.clearInterval(petsId);
      window.clearInterval(idleId);
      if (clickTimer !== undefined) {
        window.clearTimeout(clickTimer);
      }
      stopDragButtonPoller();
      clearSceneReconcileTimer();
    };
  });

  $effect(() => {
    const petSignature = activePetSignature;
    const pet = untrack(() => activePet);
    let cancelled = false;
    activeSpriteUrl = "";
    void petSignature;
    scheduledScene = initialScene();
    clearSceneReconcileTimer();

    if (!pet) {
      return;
    }

    getPetSpriteUrl(pet).then((url) => {
      if (!cancelled) {
        activeSpriteUrl = url;
      }
    });

    return () => {
      cancelled = true;
    };
  });

  function pressPet(event: PointerEvent) {
    if (event.button !== 0) {
      return;
    }

    event.preventDefault();
    pressedPointerId = event.pointerId;
    dragStartVersion += 1;
    dragCursorSampleVersion += 1;
    pressStartX = event.clientX;
    pressStartY = event.clientY;
    lastPointerScreenX = event.screenX;
    lastPointerDirectionAt = 0;
    dragMoved = false;

    if (event.currentTarget instanceof HTMLElement) {
      event.currentTarget.setPointerCapture(event.pointerId);
    }

  }

  async function movePet(event: PointerEvent) {
    if (
      pressedPointerId !== event.pointerId ||
      (event.buttons & 1) === 0
    ) {
      return;
    }

    const pointerDeltaX = consumePointerDeltaX(event);

    if (dragging) {
      applyPointerDragDirection(pointerDeltaX);
      return;
    }

    if (
      !hasDragMovement(pressStartX, pressStartY, event.clientX, event.clientY)
    ) {
      return;
    }

    const deltaX = event.clientX - pressStartX;
    dragMoved = true;

    const directionDeltaX =
      dragEventFromDelta(pointerDeltaX, null) === null ? deltaX : pointerDeltaX;
    const initialDragEvent = dragEventFromDelta(directionDeltaX, null);
    lastPointerDirectionAt = Date.now();

    if (!isTauri()) {
      if (initialDragEvent) {
        await triggerAction(initialDragEvent);
      }
      return;
    }

    beginNativeDrag(initialDragEvent);
  }

  async function releasePet(event: PointerEvent) {
    if (pressedPointerId !== event.pointerId) {
      return;
    }

    if (event.currentTarget instanceof HTMLElement) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }

    pressedPointerId = null;
    lastPointerScreenX = null;
    dragStartVersion += 1;
    dragCursorSampleVersion += 1;

    if (dragging) {
      await endDrag();
      return;
    }

    if (!dragMoved) {
      queueClickAction();
    }
  }

  async function cancelPet(event: PointerEvent) {
    if (pressedPointerId === event.pointerId) {
      pressedPointerId = null;
      lastPointerScreenX = null;
      dragStartVersion += 1;
      dragCursorSampleVersion += 1;
    }

    if (dragging) {
      await endDrag();
    }
  }

  function beginNativeDrag(initialEvent: DragActionEvent | null) {
    if (dragging) {
      return;
    }

    const startVersion = ++dragStartVersion;

    if (clickTimer !== undefined) {
      window.clearTimeout(clickTimer);
      clickTimer = undefined;
    }

    const petWindow = getCurrentWindow();
    if (
      startVersion !== dragStartVersion ||
      pressedPointerId === null ||
      dragging
    ) {
      return;
    }
    dragMoved = true;
    dragging = true;
    beginDragTracker();
    lastDragCursorX = null;
    lastDragEvent = null;
    dragCursorSampleVersion += 1;
    if (initialEvent) {
      void applyDragDirection(initialEvent);
    }
    void seedDragCursor(startVersion);
    void petWindow.startDragging().catch(() => {
      if (startVersion === dragStartVersion && dragging) {
        void endDrag();
      }
    });
  }

  async function seedDragCursor(startVersion: number) {
    const cursor = await cursorPosition().catch(() => null);
    if (cursor && startVersion === dragStartVersion && dragging) {
      lastDragCursorX = cursor.x;
    }
  }

  async function endDrag() {
    if (!dragging) {
      return;
    }

    stopDragTracker();
    stopDragButtonPoller();
    dragStartVersion += 1;
    dragCursorSampleVersion += 1;
    dragging = false;
    pressedPointerId = null;
    lastPointerScreenX = null;
    if (isTauri()) {
      const position = await getCurrentWindow().outerPosition();
      await setPetWindowPosition({ x: position.x, y: position.y });
    }
    lastDragEvent = null;
  }

  function queueClickAction() {
    const clickAction = nextClickAction(clickTimer !== undefined);

    if (clickTimer !== undefined) {
      window.clearTimeout(clickTimer);
      clickTimer = undefined;
      void triggerAction(clickAction);
      return;
    }

    clickTimer = window.setTimeout(() => {
      clickTimer = undefined;
      void triggerAction(clickAction);
    }, DOUBLE_CLICK_MS);
  }

  async function keyPet(event: KeyboardEvent) {
    if (event.key !== "Enter" && event.key !== " ") {
      return;
    }

    event.preventDefault();
    await triggerAction("click");
  }

  async function openFocusPanel(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();

    if (clickTimer !== undefined) {
      window.clearTimeout(clickTimer);
      clickTimer = undefined;
    }

    if (!isTauri()) {
      return;
    }

    const focusPanel = await Window.getByLabel("focus-panel");
    if (!focusPanel) {
      return;
    }

    const position = visibleLogicalWindowPosition(
      event.screenX,
      event.screenY,
      FOCUS_PANEL_WIDTH,
      FOCUS_PANEL_HEIGHT,
    );
    await showFocusPanelLauncher(focusPanel, position);
  }

  async function showFocusPanelLauncher(
    focusPanel: Window,
    position: { x: number; y: number },
  ) {
    await focusPanel.hide();
    await focusPanel.setSize(
      new LogicalSize(FOCUS_LAUNCHER_WIDTH, FOCUS_LAUNCHER_HEIGHT),
    );
    await focusPanel.setPosition(new LogicalPosition(position.x, position.y));
    await focusPanel.show();
    await focusPanel.setFocus();
    await focusPanel.emitTo("focus-panel", "focus-panel:launcher");
    window.setTimeout(() => {
      void focusPanel.setSize(
        new LogicalSize(FOCUS_LAUNCHER_WIDTH, FOCUS_LAUNCHER_HEIGHT),
      );
      void focusPanel.emitTo("focus-panel", "focus-panel:launcher");
    }, 80);
  }

  function visibleLogicalWindowPosition(
    x: number,
    y: number,
    width: number,
    height: number,
  ) {
    const screen = window.screen as Screen & {
      availLeft?: number;
      availTop?: number;
    };
    const minX = screen.availLeft ?? 0;
    const minY = screen.availTop ?? 0;
    const maxX = minX + screen.availWidth - width;
    const maxY = minY + screen.availHeight - height;

    return {
      x: clamp(Math.round(x), minX, Math.max(minX, maxX)),
      y: clamp(Math.round(y), minY, Math.max(minY, maxY)),
    };
  }

  function beginDragTracker() {
    stopDragTracker();
    dragTracker = window.setInterval(trackDragDirection, 80);
    beginDragButtonPoller();
  }

  function stopDragTracker() {
    if (dragTracker !== undefined) {
      window.clearInterval(dragTracker);
      dragTracker = undefined;
    }

    dragCursorSampleVersion += 1;
    lastDragCursorX = null;
    lastDragEvent = null;
  }

  function beginDragButtonPoller() {
    stopDragButtonPoller();
    dragButtonPoller = window.setInterval(() => {
      void pollDragButton();
    }, 80);
  }

  function stopDragButtonPoller() {
    if (dragButtonPoller !== undefined) {
      window.clearInterval(dragButtonPoller);
      dragButtonPoller = undefined;
    }
  }

  async function pollDragButton() {
    if (!dragging || !isTauri()) {
      return;
    }

    if (!(await isLeftMouseButtonPressed())) {
      await endDrag();
    }
  }

  async function trackDragDirection() {
    if (!isTauri() || lastDragCursorX === null) {
      return;
    }

    const cursor = await latestDragCursor();
    if (!cursor) {
      return;
    }
    await trackDragPosition(cursor.x);
  }

  async function latestDragCursor() {
    const sampleVersion = ++dragCursorSampleVersion;
    const cursor = await cursorPosition();

    if (sampleVersion !== dragCursorSampleVersion || !dragging) {
      return null;
    }

    return cursor;
  }

  async function trackDragPosition(currentCursorX: number) {
    if (!dragging || lastDragCursorX === null) {
      return;
    }

    const previousCursorX = lastDragCursorX;
    lastDragCursorX = currentCursorX;
    const deltaX = currentCursorX - previousCursorX;

    if (Date.now() - lastPointerDirectionAt < DRAG_POINTER_DIRECTION_GRACE_MS) {
      return;
    }

    const nextEvent = dragEventFromDelta(deltaX, lastDragEvent);
    if (nextEvent) {
      await applyDragDirection(nextEvent);
    }
  }

  function consumePointerDeltaX(event: PointerEvent) {
    const screenDeltaX =
      lastPointerScreenX === null ? 0 : event.screenX - lastPointerScreenX;
    lastPointerScreenX = event.screenX;

    return Math.abs(event.movementX) >= Math.abs(screenDeltaX)
      ? event.movementX
      : screenDeltaX;
  }

  function applyPointerDragDirection(deltaX: number) {
    const nextEvent = dragEventFromDelta(deltaX, lastDragEvent);
    if (!nextEvent || nextEvent === lastDragEvent) {
      return;
    }

    lastPointerDirectionAt = Date.now();
    void applyDragDirection(nextEvent);
  }

  async function applyDragDirection(event: DragActionEvent) {
    dragMoved = true;

    if (event !== lastDragEvent) {
      lastDragEvent = event;
      const resolvedScene = resolvePetAction(activePet?.actionMap, event);
      await applyActionScene(
        event,
        "movement",
        repeatedAnimationDurationMs(resolvedScene, 3),
        resolvedScene,
      );
    }
  }

  async function triggerAction(event: PetActionEvent) {
    const resolvedScene = resolvePetAction(activePet?.actionMap, event);
    await applyActionScene(
      event,
      "interaction",
      repeatedAnimationDurationMs(resolvedScene, 3),
      resolvedScene,
    );
  }

  async function applyActionScene(
    event: PetActionEvent,
    source: SceneSource,
    minDurationMs?: number,
    resolvedScene = resolvePetAction(activePet?.actionMap, event),
  ) {
    const now = Date.now();
    lastInteraction = now;
    const eventTimestamp = Math.max(now, (scheduledScene.timestamp ?? 0) + 1);
    const sceneEvent: SceneEvent = {
      ...actionSceneEvent(
        activePet?.actionMap,
        event,
        source,
        eventTimestamp,
        minDurationMs,
      ),
      state: resolvedScene,
    };
    scheduledScene = selectScene(
      scheduledScene,
      [...ambientSceneEvents(now), sceneEvent],
      now,
    );
    scenePlaybackKey += 1;
    scheduleSceneReconcile(now);
    await commitScene(scheduledScene.state);
  }

  async function applyIdleScene() {
    await reconcileScene();
  }

  async function reconcileScene() {
    if (dragging || !activePet) {
      return;
    }

    const now = Date.now();
    if (
      runtime.focusState.status === "running" &&
      runtime.focusState.endsAt &&
      runtime.focusState.endsAt <= now
    ) {
      runtime = await setFocusState({
        mode: runtime.focusState.mode,
        status: "complete",
        lastEvent:
          runtime.focusState.mode === "break"
            ? "break-complete"
            : "focus-complete",
        remainingMs: 0,
        endsAt: null,
      });
    }
    scheduledScene = selectScene(scheduledScene, ambientSceneEvents(now), now);
    scheduleSceneReconcile(now);
    if (scheduledScene.state !== runtime.scene) {
      await commitScene(scheduledScene.state);
    }
  }

  function scheduleSceneReconcile(now: number) {
    clearSceneReconcileTimer();

    if (scheduledScene.lockedUntil <= now) {
      return;
    }

    sceneReconcileTimer = window.setTimeout(() => {
      sceneReconcileTimer = undefined;
      void reconcileScene();
    }, scheduledScene.lockedUntil - now + 10);
  }

  function clearSceneReconcileTimer() {
    if (sceneReconcileTimer !== undefined) {
      window.clearTimeout(sceneReconcileTimer);
      sceneReconcileTimer = undefined;
    }
  }

  function ambientSceneEvents(now: number) {
    return runtimeSceneEvents(
      runtime,
      activePet?.actionMap,
      now,
      now - lastInteraction,
    );
  }

  async function commitScene(scene: RuntimeState["scene"]) {
    const version = ++sceneUpdateVersion;
    pendingSceneUpdates += 1;
    runtime = { ...runtime, scene };

    sceneUpdateQueue = sceneUpdateQueue
      .catch(() => undefined)
      .then(async () => {
        const nextRuntime = await setScene(scene);
        if (version === sceneUpdateVersion) {
          runtime = nextRuntime;
        }
      })
      .finally(() => {
        pendingSceneUpdates = Math.max(0, pendingSceneUpdates - 1);
      });

    await sceneUpdateQueue;
  }
</script>

<button
  type="button"
  class="pet-window"
  onpointerdown={pressPet}
  onpointermove={movePet}
  onpointerup={releasePet}
  onpointercancel={cancelPet}
  oncontextmenu={openFocusPanel}
  onkeydown={keyPet}
>
  {#if activePet && activeSpriteUrl}
    <SpritePet
      imageUrl={activeSpriteUrl}
      state={runtime.scene}
      scale={runtime.scale}
      playbackKey={scenePlaybackKey}
    />
  {/if}
</button>

<style>
  .pet-window {
    width: 192px;
    height: 208px;
    min-height: 208px;
    display: grid;
    place-items: start;
    border: 0;
    padding: 0;
    background: transparent;
    cursor: grab;
    transition: none;
  }

  .pet-window:active {
    cursor: grabbing;
    transform: none;
  }
</style>
