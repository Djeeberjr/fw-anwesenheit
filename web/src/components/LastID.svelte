<script lang="ts">
  import { onMount } from "svelte";

  let { onAdd }: { onAdd?: (id: string) => void } = $props();

  let id = $state("");
  let flashing = $state(false);

  onMount(() => {
    let sse = new EventSource("/api/idevent");
    sse.addEventListener("msg", function (e) {
      const newID = e.data;

      if (id != newID) {
        flashing = true;

        setTimeout(() => {
          flashing = false;
        }, 1100);
      }
      id = newID;
    });
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
    animation: flash-green 1.2s;
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
