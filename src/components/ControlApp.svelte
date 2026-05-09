<script lang="ts">
  import SpritePet from "./SpritePet.svelte";
  import { ANIMATION_ROWS, type PetAnimationState } from "../lib/animations";
  import {
    chooseImportPath,
    getPetSpriteUrl,
    getRuntimeState,
    importPetFromPath,
    importPetdex,
    listPets,
    scanCodexPets,
    setActivePet,
    setScale,
    setScene,
    type PetInfo,
    type RuntimeState,
  } from "../lib/petpop";

  let pets = $state<PetInfo[]>([]);
  let runtime = $state<RuntimeState>({
    activePetId: null,
    scene: "idle",
    scale: 1,
  });
  let importPath = $state("");
  let petdexInput = $state("");
  let status = $state("Ready");
  let busy = $state(false);
  let taskRunning = $state(false);
  let lastInteraction = Date.now();
  let activeSpriteUrl = $state("");

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

    if (!nextRuntime.activePetId && nextPets[0]) {
      runtime = await setActivePet(nextPets[0].id);
    }
  }

  async function runAction<T>(message: string, action: () => Promise<T>) {
    busy = true;
    status = message;

    try {
      await action();
      status = "Done";
      await setScene("jumping");
      await refresh();
    } catch (error) {
      status = error instanceof Error ? error.message : String(error);
      await setScene("failed");
      await refresh();
    } finally {
      busy = false;
    }
  }

  async function selectPet(id: string) {
    runtime = await setActivePet(id);
    await setScene("waving");
    await refresh();
  }

  async function importPathNow() {
    if (!importPath.trim()) {
      status = "Choose a package or folder path";
      return;
    }

    await runAction("Importing", async () => {
      await importPetFromPath(importPath.trim());
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
      status = "Enter a PetDex id or URL";
      return;
    }

    await runAction("Importing from PetDex", async () => {
      await importPetdex(petdexInput.trim());
      petdexInput = "";
    });
  }

  async function scanCodexNow() {
    await runAction("Scanning Codex pets", async () => {
      await scanCodexPets();
    });
  }

  async function triggerScene(state: PetAnimationState) {
    lastInteraction = Date.now();
    runtime = await setScene(state);
  }

  async function toggleTask() {
    taskRunning = !taskRunning;
    await triggerScene(taskRunning ? "running" : "idle");
  }

  async function updateScale(value: number) {
    runtime = await setScale(value);
  }

  $effect(() => {
    refresh();
    const id = window.setInterval(async () => {
      if (taskRunning) {
        await setScene("running");
        await refresh();
        return;
      }

      if (Date.now() - lastInteraction > 60_000) {
        await setScene("waiting");
        await refresh();
      }
    }, 5000);

    return () => window.clearInterval(id);
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
</script>

<main class="app-shell">
  <section class="library">
    <div class="brand">
      <span class="mark"></span>
      <div>
        <h1>PetPop</h1>
        <p>Codex pet runtime</p>
      </div>
    </div>

    <div class="pet-list" aria-label="Pet library">
      {#each pets as pet}
        <button
          class:active={pet.id === activePet?.id}
          onclick={() => selectPet(pet.id)}
        >
          <span>{pet.displayName}</span>
          <small>{pet.sourceKind}</small>
        </button>
      {/each}
    </div>
  </section>

  <section
    class="workspace"
    role="application"
    onmousemove={() => (lastInteraction = Date.now())}
  >
    <header class="topbar">
      <div>
        <h2>{activePet?.displayName ?? "No pet"}</h2>
        <p>{activePet?.description ?? "Import a Codex-compatible pet package."}</p>
      </div>
      <div class="status" class:busy>{status}</div>
    </header>

    <div class="stage">
      {#if activePet && activeSpriteUrl}
        <SpritePet
          imageUrl={activeSpriteUrl}
          state={runtime.scene}
          scale={runtime.scale}
        />
      {:else if activePet}
        <div class="stage-message">Loading sprite</div>
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
        <span>Scale</span>
        <input
          type="range"
          min="0.5"
          max="2"
          step="0.05"
          value={runtime.scale}
          oninput={(event) =>
            updateScale(Number((event.target as HTMLInputElement).value))}
        />
        <strong>{runtime.scale.toFixed(2)}x</strong>
      </label>

      <button class:active={taskRunning} onclick={toggleTask}>
        {taskRunning ? "Stop task" : "Start task"}
      </button>
    </div>
  </section>

  <aside class="importer">
    <h2>Import</h2>

    <label>
      <span>Package path</span>
      <div class="input-row">
        <input bind:value={importPath} placeholder="zip or pet folder" />
        <button onclick={() => pickPath("file")}>Zip</button>
        <button onclick={() => pickPath("folder")}>Folder</button>
      </div>
    </label>
    <button disabled={busy} onclick={importPathNow}>Import local</button>

    <label>
      <span>PetDex</span>
      <input bind:value={petdexInput} placeholder="boba or PetDex URL" />
    </label>
    <button disabled={busy} onclick={importPetdexNow}>Import PetDex</button>

    <button disabled={busy} onclick={scanCodexNow}>Scan Codex pets</button>

    <div class="source-note">
      PetDex pets are user-submitted fan art. PetPop stores only pets you
      import.
    </div>
  </aside>
</main>
