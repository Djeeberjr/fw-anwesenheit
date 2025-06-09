<script lang="ts">
  let { id, onAdd }: { id: string; onAdd?: (id: string) => void } = $props();

  let lastID = id;
  let flashing = $state(false);

  $effect(() => {
    if (lastID != id) {
      flashing = true;

      setTimeout(() => {
        flashing = false;
      }, 1100);
    }
    lastID = id;
  });
</script>

<div class=" text-xl text-center">
  Letzte ID
  <div class="flex justify-center">
    <span
      class="{flashing
        ? 'flash'
        : ''} font-bold rounded-md px-1 font-mono min-w-36">{id}</span
    >
    <button
      class="bg-indigo-500 rounded-2xl px-2 cursor-pointer mx-2"
      onclick={() => {
        if (onAdd) {
          onAdd(id);
        }
      }}>+</button
    >
  </div>
</div>

<style scoped>
  .flash {
    animation: flash-green 1s;
  }

  @keyframes flash-green {
    0% {
      background-color: transparent;
    }
    40% {
      background-color: oklch(59.6% 0.145 163.225);
    }
    60% {
      background-color: oklch(59.6% 0.145 163.225);
    }
    100% {
      background-color: transparent;
    }
  }
</style>
