<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import ControlApp from "./components/ControlApp.svelte";
  import FocusPanel from "./components/FocusPanel.svelte";
  import PetWindow from "./components/PetWindow.svelte";
  import { isTauri } from "./lib/petpop";

  let windowLabel = $state("main");

  $effect(() => {
    if (isTauri()) {
      windowLabel = getCurrentWindow().label;
    }
  });
</script>

{#if windowLabel === "pet"}
  <PetWindow />
{:else if windowLabel === "focus-panel"}
  <FocusPanel />
{:else}
  <ControlApp />
{/if}
