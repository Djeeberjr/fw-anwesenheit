<script lang="ts">
  import Cleanup from "./Cleanup.svelte";
  import TimeSync from "./TimeSync.svelte";
  import Console from "./Console.svelte";

  let showMaintain = $state(false);
</script>

<button onclick={() => (showMaintain = !showMaintain)}>
  <div class="flex justify-start">
    <span class="m-auto mr-1"
      >{#if showMaintain}▼{:else}▶{/if}</span
    >
    <h1 class="text-center text-3xl sm:text-4xl font-bold text-gray-800">
      Wartung
    </h1>
  </div>
</button>
{#if showMaintain}
  <TimeSync />
  <Cleanup />
  <Console />
{/if}
