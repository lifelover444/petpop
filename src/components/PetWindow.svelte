<script lang="ts">
  import { PhysicalPosition } from "@tauri-apps/api/dpi";
  import {
    availableMonitors,
    cursorPosition,
    getCurrentWindow,
    type Monitor,
  } from "@tauri-apps/api/window";
  import SpritePet from "./SpritePet.svelte";
  import {
    resolvePetAction,
    type PetActionEvent,
  } from "../lib/actions";
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
    isTauri,
    listPets,
    setFocusState,
    setPetWindowPosition,
    setScene,
  } from "../lib/petpop";

  type DragActionEvent = Extract<PetActionEvent, "drag-left" | "drag-right">;

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
  let pressCursorX: number | null = null;
  let pressCursorY: number | null = null;
  let pressWindowX: number | null = null;
  let pressWindowY: number | null = null;
  let dragCursorStartX = 0;
  let dragCursorStartY = 0;
  let dragWindowStartX = 0;
  let dragWindowStartY = 0;
  let usingNativeDragFallback = false;
  let clickTimer: number | undefined;
  let dragTracker: number | undefined;
  let dragEndFallback: number | undefined;
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
  let sceneUpdateVersion = 0;
  let pendingSceneUpdates = 0;
  let sceneUpdateQueue = Promise.resolve();
  const DRAG_THRESHOLD_PX = 5;
  const DRAG_DIRECTION_MIN_DELTA_PX = 1;
  const DRAG_POINTER_DIRECTION_GRACE_MS = 120;
  const DOUBLE_CLICK_MS = 260;
  const DRAG_RELEASE_FALLBACK_MS = 900;
  const PET_WINDOW_WIDTH = 192;
  const PET_WINDOW_HEIGHT = 208;

  const activePet = $derived(
    pets.find((pet) => pet.id === runtime.activePetId) ?? pets[0],
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
    const monitors = await availableMonitors();
    const monitor =
      monitors.find((item) => intersectsMonitor(x, y, item.workArea)) ??
      monitors[0];

    if (!monitor) {
      return { x, y };
    }

    const minX = monitor.workArea.position.x;
    const minY = monitor.workArea.position.y;
    const maxX = monitor.workArea.position.x + monitor.workArea.size.width - PET_WINDOW_WIDTH;
    const maxY = monitor.workArea.position.y + monitor.workArea.size.height - PET_WINDOW_HEIGHT;

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
      clearDragEndFallback();
    };
  });

  $effect(() => {
    const pet = activePet;
    let cancelled = false;
    activeSpriteUrl = "";
    JSON.stringify(pet?.actionMap ?? {});
    scheduledScene = initialScene();

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

  async function pressPet(event: PointerEvent) {
    if (event.button !== 0) {
      return;
    }

    event.preventDefault();
    pressedPointerId = event.pointerId;
    dragStartVersion += 1;
    dragCursorSampleVersion += 1;
    pressStartX = event.clientX;
    pressStartY = event.clientY;
    pressCursorX = null;
    pressCursorY = null;
    pressWindowX = null;
    pressWindowY = null;
    lastPointerScreenX = event.screenX;
    lastPointerDirectionAt = 0;
    dragMoved = false;

    if (event.currentTarget instanceof HTMLElement) {
      event.currentTarget.setPointerCapture(event.pointerId);
    }

    if (isTauri()) {
      try {
        const [cursor, position] = await Promise.all([
          cursorPosition(),
          getCurrentWindow().outerPosition(),
        ]);
        if (pressedPointerId === event.pointerId) {
          pressCursorX = cursor.x;
          pressCursorY = cursor.y;
          pressWindowX = position.x;
          pressWindowY = position.y;
        }
      } catch {
        pressCursorX = null;
        pressCursorY = null;
        pressWindowX = null;
        pressWindowY = null;
      }
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
      void updateManualDrag();
      return;
    }

    const deltaX = event.clientX - pressStartX;
    const deltaY = event.clientY - pressStartY;

    if (Math.hypot(deltaX, deltaY) < DRAG_THRESHOLD_PX) {
      return;
    }

    dragMoved = true;

    const directionDeltaX =
      Math.abs(pointerDeltaX) >= DRAG_DIRECTION_MIN_DELTA_PX
        ? pointerDeltaX
        : deltaX;
    lastPointerDirectionAt = Date.now();

    if (!isTauri()) {
      await triggerAction(directionDeltaX > 0 ? "drag-right" : "drag-left");
      return;
    }

    await beginManualDrag(directionDeltaX);
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

  async function beginManualDrag(initialDeltaX: number) {
    if (dragging) {
      return;
    }

    const startVersion = ++dragStartVersion;

    if (clickTimer !== undefined) {
      window.clearTimeout(clickTimer);
      clickTimer = undefined;
    }

    try {
      const cursor = await cursorPosition();
      const position = await getCurrentWindow().outerPosition();
      if (
        startVersion !== dragStartVersion ||
        pressedPointerId === null ||
        dragging
      ) {
        return;
      }
      dragCursorStartX = pressCursorX ?? cursor.x;
      dragCursorStartY = pressCursorY ?? cursor.y;
      dragWindowStartX = pressWindowX ?? position.x;
      dragWindowStartY = pressWindowY ?? position.y;
      dragMoved = true;
      dragging = true;
      usingNativeDragFallback = false;
      beginDragTracker();
      lastDragCursorX = cursor.x;
      lastDragEvent = null;
      dragCursorSampleVersion += 1;
      void triggerAction("drag-start", 0);
      void applyDragDirection(initialDeltaX > 0 ? "drag-right" : "drag-left");
      await moveWindowToCursor(cursor.x, cursor.y);
    } catch {
      if (
        startVersion === dragStartVersion &&
        pressedPointerId !== null &&
        !dragging
      ) {
        await beginNativeDragFallback(initialDeltaX, startVersion);
      }
    }
  }

  async function beginNativeDragFallback(
    initialDeltaX: number,
    startVersion: number,
  ) {
    const petWindow = getCurrentWindow();
    const cursor = await cursorPosition();
    if (
      startVersion !== dragStartVersion ||
      pressedPointerId === null ||
      dragging
    ) {
      return;
    }
    dragMoved = true;
    dragging = true;
    usingNativeDragFallback = true;
    beginDragTracker();
    lastDragCursorX = cursor.x;
    lastDragEvent = null;
    dragCursorSampleVersion += 1;
    void triggerAction("drag-start", 0);
    void applyDragDirection(initialDeltaX > 0 ? "drag-right" : "drag-left");
    scheduleDragEndFallback();
    await petWindow.startDragging();
  }

  async function endDrag() {
    if (!dragging) {
      return;
    }

    stopDragTracker();
    clearDragEndFallback();
    dragStartVersion += 1;
    dragCursorSampleVersion += 1;
    dragging = false;
    usingNativeDragFallback = false;
    pressedPointerId = null;
    lastPointerScreenX = null;
    if (isTauri()) {
      const position = await getCurrentWindow().outerPosition();
      await setPetWindowPosition({ x: position.x, y: position.y });
    }
    await triggerAction("drag-end", 250);
  }

  function queueClickAction() {
    if (clickTimer !== undefined) {
      window.clearTimeout(clickTimer);
      clickTimer = undefined;
      void triggerAction("double-click");
      return;
    }

    clickTimer = window.setTimeout(() => {
      clickTimer = undefined;
      void triggerAction("click");
    }, DOUBLE_CLICK_MS);
  }

  async function keyPet(event: KeyboardEvent) {
    if (event.key !== "Enter" && event.key !== " ") {
      return;
    }

    event.preventDefault();
    await triggerAction("click");
  }

  function beginDragTracker() {
    stopDragTracker();
    dragTracker = window.setInterval(trackDragDirection, 80);
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

  function scheduleDragEndFallback() {
    clearDragEndFallback();
    dragEndFallback = window.setTimeout(() => {
      dragEndFallback = undefined;
      if (dragging) {
        void endDrag();
      }
    }, DRAG_RELEASE_FALLBACK_MS);
  }

  function clearDragEndFallback() {
    if (dragEndFallback !== undefined) {
      window.clearTimeout(dragEndFallback);
      dragEndFallback = undefined;
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
    if (dragging && !usingNativeDragFallback) {
      await moveWindowToCursor(cursor.x, cursor.y);
    }
    await trackDragPosition(cursor.x);
  }

  async function updateManualDrag() {
    if (!isTauri() || usingNativeDragFallback) {
      return;
    }

    const cursor = await latestDragCursor();
    if (!cursor) {
      return;
    }
    await moveWindowToCursor(cursor.x, cursor.y);
  }

  async function latestDragCursor() {
    const sampleVersion = ++dragCursorSampleVersion;
    const cursor = await cursorPosition();

    if (sampleVersion !== dragCursorSampleVersion || !dragging) {
      return null;
    }

    return cursor;
  }

  async function moveWindowToCursor(cursorX: number, cursorY: number) {
    const nextX = Math.round(dragWindowStartX + cursorX - dragCursorStartX);
    const nextY = Math.round(dragWindowStartY + cursorY - dragCursorStartY);
    await getCurrentWindow().setPosition(new PhysicalPosition(nextX, nextY));
  }

  async function trackDragPosition(currentCursorX: number) {
    if (!dragging || lastDragCursorX === null) {
      return;
    }

    const previousCursorX = lastDragCursorX;
    lastDragCursorX = currentCursorX;
    const deltaX = currentCursorX - previousCursorX;

    if (Math.abs(deltaX) < 2) {
      return;
    }

    if (Date.now() - lastPointerDirectionAt < DRAG_POINTER_DIRECTION_GRACE_MS) {
      return;
    }

    if (usingNativeDragFallback) {
      scheduleDragEndFallback();
    }
    await applyDragDirection(deltaX > 0 ? "drag-right" : "drag-left");
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
    if (Math.abs(deltaX) < DRAG_DIRECTION_MIN_DELTA_PX) {
      return;
    }

    lastPointerDirectionAt = Date.now();
    void applyDragDirection(deltaX > 0 ? "drag-right" : "drag-left");
  }

  async function applyDragDirection(event: DragActionEvent) {
    dragMoved = true;

    if (event !== lastDragEvent) {
      lastDragEvent = event;
      await applyActionScene(
        event,
        "movement",
        0,
        resolvePetAction(activePet?.actionMap, event),
      );
    }
  }

  async function triggerAction(event: PetActionEvent, minDurationMs = 900) {
    await applyActionScene(event, "interaction", minDurationMs);
  }

  async function applyActionScene(
    event: PetActionEvent,
    source: SceneSource,
    minDurationMs?: number,
    resolvedScene = resolvePetAction(activePet?.actionMap, event),
  ) {
    lastInteraction = Date.now();
    const now = Date.now();
    const sceneEvent: SceneEvent = {
      ...actionSceneEvent(activePet?.actionMap, event, source, now, minDurationMs),
      state: resolvedScene,
    };
    scheduledScene = selectScene(
      scheduledScene,
      [...ambientSceneEvents(now), sceneEvent],
      now,
    );
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
    if (scheduledScene.state !== runtime.scene) {
      await commitScene(scheduledScene.state);
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
  onkeydown={keyPet}
>
  {#if activePet && activeSpriteUrl}
    <SpritePet
      imageUrl={activeSpriteUrl}
      state={runtime.scene}
      scale={runtime.scale}
    />
  {/if}
</button>

<style>
  .pet-window {
    width: fit-content;
    height: fit-content;
    min-height: 0;
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
