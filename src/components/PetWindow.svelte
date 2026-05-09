<script lang="ts">
  import { LogicalPosition } from "@tauri-apps/api/dpi";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import SpritePet from "./SpritePet.svelte";
  import type { PetInfo, RuntimeState } from "../lib/petpop";
  import {
    getRuntimeState,
    getPetSpriteUrl,
    getPetWindowPosition,
    isTauri,
    listPets,
    setPetWindowPosition,
    setScene,
  } from "../lib/petpop";

  let pets = $state<PetInfo[]>([]);
  let runtime = $state<RuntimeState>({
    activePetId: null,
    scene: "idle",
    scale: 1,
  });
  let activeSpriteUrl = $state("");
  let dragTracker: number | undefined;
  let lastDragX: number | null = null;
  let lastDragScene: RuntimeState["scene"] | null = null;

  const activePet = $derived(
    pets.find((pet) => pet.id === runtime.activePetId) ?? pets[0],
  );

  async function refreshPets() {
    pets = await listPets();
  }

  async function refreshRuntime() {
    runtime = await getRuntimeState();
  }

  $effect(() => {
    document.documentElement.classList.add("pet-runtime");
    document.body.classList.add("pet-runtime");
    if (isTauri()) {
      getPetWindowPosition().then((position) => {
        getCurrentWindow().setPosition(new LogicalPosition(position.x, position.y));
      });
    }
    refreshPets();
    refreshRuntime();
    const runtimeId = window.setInterval(refreshRuntime, 350);
    const petsId = window.setInterval(refreshPets, 5000);
    return () => {
      document.documentElement.classList.remove("pet-runtime");
      document.body.classList.remove("pet-runtime");
      window.clearInterval(runtimeId);
      window.clearInterval(petsId);
    };
  });

  $effect(() => {
    const pet = activePet;
    let cancelled = false;
    activeSpriteUrl = "";

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

  async function beginDrag(event: PointerEvent) {
    if (event.button !== 0) {
      return;
    }

    if (!isTauri()) {
      await setScene("waving");
      return;
    }

    const window = getCurrentWindow();
    const position = await window.outerPosition();
    lastDragX = position.x;
    lastDragScene = null;
    beginDragTracker();
    await setScene("waving");
    await window.startDragging();
  }

  async function endDrag() {
    stopDragTracker();
    if (isTauri()) {
      const position = await getCurrentWindow().outerPosition();
      await setPetWindowPosition({ x: position.x, y: position.y });
    }
    await setScene("idle");
  }

  function beginDragTracker() {
    stopDragTracker();
    dragTracker = window.setInterval(trackDragDirection, 80);
  }

  function stopDragTracker() {
    if (dragTracker) {
      window.clearInterval(dragTracker);
      dragTracker = undefined;
    }

    lastDragX = null;
    lastDragScene = null;
  }

  async function trackDragDirection() {
    if (!isTauri() || lastDragX === null) {
      return;
    }

    const position = await getCurrentWindow().outerPosition();
    const deltaX = position.x - lastDragX;

    if (Math.abs(deltaX) < 2) {
      return;
    }

    const nextScene = deltaX > 0 ? "running-right" : "running-left";
    lastDragX = position.x;

    if (nextScene !== lastDragScene) {
      lastDragScene = nextScene;
      await setScene(nextScene);
      runtime = await getRuntimeState();
    }
  }
</script>

<main
  class="pet-window"
  onpointerdown={beginDrag}
  onpointerup={endDrag}
  onpointercancel={endDrag}
  ondblclick={() => setScene("jumping")}
>
  {#if activePet && activeSpriteUrl}
    <SpritePet
      imageUrl={activeSpriteUrl}
      state={runtime.scene}
      scale={runtime.scale}
    />
  {/if}
</main>

<style>
  .pet-window {
    width: fit-content;
    height: fit-content;
    background: transparent;
    cursor: grab;
  }

  .pet-window:active {
    cursor: grabbing;
  }
</style>
