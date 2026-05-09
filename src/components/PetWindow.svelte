<script lang="ts">
  import { LogicalPosition } from "@tauri-apps/api/dpi";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import SpritePet from "./SpritePet.svelte";
  import type { PetInfo, RuntimeState } from "../lib/petdesk";
  import {
    getRuntimeState,
    isTauri,
    listPets,
    setScene,
    spriteUrl,
  } from "../lib/petdesk";

  let pets = $state<PetInfo[]>([]);
  let runtime = $state<RuntimeState>({
    activePetId: null,
    scene: "idle",
    scale: 1,
  });
  let dragging = false;
  let dragStart: { x: number; y: number; winX: number; winY: number } | null =
    null;

  const activePet = $derived(
    pets.find((pet) => pet.id === runtime.activePetId) ?? pets[0],
  );

  async function refresh() {
    const [nextPets, nextRuntime] = await Promise.all([
      listPets(),
      getRuntimeState(),
    ]);
    pets = nextPets;
    runtime = nextRuntime;
  }

  $effect(() => {
    refresh();
    const id = window.setInterval(refresh, 300);
    return () => window.clearInterval(id);
  });

  async function beginDrag(event: PointerEvent) {
    dragging = true;
    await setScene("waving");

    if (!isTauri()) {
      return;
    }

    const position = await getCurrentWindow().outerPosition();
    dragStart = {
      x: event.screenX,
      y: event.screenY,
      winX: position.x,
      winY: position.y,
    };
  }

  async function moveDrag(event: PointerEvent) {
    if (!dragging || !dragStart || !isTauri()) {
      return;
    }

    const dx = event.screenX - dragStart.x;
    const dy = event.screenY - dragStart.y;
    const state = dx >= 0 ? "running-right" : "running-left";
    await setScene(state);
    await getCurrentWindow().setPosition(
      new LogicalPosition(dragStart.winX + dx, dragStart.winY + dy),
    );
  }

  async function endDrag() {
    dragging = false;
    dragStart = null;
    await setScene("idle");
  }
</script>

<main
  class="pet-window"
  onpointerdown={beginDrag}
  onpointermove={moveDrag}
  onpointerup={endDrag}
  onpointercancel={endDrag}
  ondblclick={() => setScene("jumping")}
>
  {#if activePet}
    <SpritePet
      imageUrl={spriteUrl(activePet.spritesheetPath)}
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
