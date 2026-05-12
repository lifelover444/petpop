<script lang="ts">
  import {
    ANIMATION_BY_STATE,
    CELL_HEIGHT,
    CELL_WIDTH,
    nextFrameDelay,
    type PetAnimationState,
  } from "../lib/animations";

  let {
    imageUrl,
    state: animationState = "idle",
    scale = 0.5,
    paused = false,
    playbackKey = 0,
  }: {
    imageUrl: string;
    state?: PetAnimationState;
    scale?: number;
    paused?: boolean;
    playbackKey?: number;
  } = $props();

  let frame = $state(0);
  let timer: number | undefined;

  $effect(() => {
    void animationState;
    void playbackKey;
    frame = 0;
  });

  $effect(() => {
    if (timer) {
      window.clearTimeout(timer);
    }

    if (paused) {
      return;
    }

    timer = window.setTimeout(() => {
      const row = ANIMATION_BY_STATE[animationState];
      frame = (frame + 1) % row.frames;
    }, nextFrameDelay(animationState, frame));

    return () => {
      if (timer) {
        window.clearTimeout(timer);
      }
    };
  });

  const row = $derived(ANIMATION_BY_STATE[animationState]);
  const backgroundPosition = $derived(
    `${-frame * CELL_WIDTH}px ${-row.row * CELL_HEIGHT}px`,
  );
  const displayWidth = $derived(CELL_WIDTH * scale);
  const displayHeight = $derived(CELL_HEIGHT * scale);
</script>

<div
  class="sprite-shell"
  style:width={`${displayWidth}px`}
  style:height={`${displayHeight}px`}
>
  <div
    class="sprite"
    aria-label={row.label}
    style:background-image={`url("${imageUrl}")`}
    style:background-position={backgroundPosition}
    style:transform={`scale(${scale})`}
  ></div>
</div>

<style>
  .sprite-shell {
    overflow: hidden;
    touch-action: none;
  }

  .sprite {
    width: 192px;
    height: 208px;
    background-repeat: no-repeat;
    background-size: 1536px 1872px;
    image-rendering: pixelated;
    transform-origin: top left;
    user-select: none;
  }
</style>
