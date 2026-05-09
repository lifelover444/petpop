<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import ControlApp from "./components/ControlApp.svelte";
  import PetWindow from "./components/PetWindow.svelte";
  import { isTauri } from "./lib/petdesk";

  let windowLabel = $state("main");

  $effect(() => {
    if (isTauri()) {
      windowLabel = getCurrentWindow().label;
    }
  });
</script>

{#if windowLabel === "pet"}
  <PetWindow />
{:else}
  <ControlApp />
{/if}
