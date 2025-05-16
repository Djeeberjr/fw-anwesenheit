<script lang="ts">
  import { onMount } from "svelte";
  import type { IDMapping } from "./IDMapping";
  let data: IDMapping | undefined = $state();

  let rows = $derived(
    data
      ? Object.entries(data.id_map).map(([id, value]) => ({
          id,
          ...value,
        }))
      : [],
  );

  onMount(async () => {
    let res = await fetch("/api/mapping");

    data = await res.json();
  });
</script>

{#if data == null}
  Loading
{:else}
  <div class="bg-indigo-500 p-2 rounded-2xl overflow-x-auto">
    <table>
      <thead>
        <tr>
          <th class="text-left pr-5">ID</th>
          <th class="text-left pr-5">Nachname</th>
          <th class="text-left pr-5">Vorname</th>
        </tr>
      </thead>
      <tbody class="">
        {#each rows as row}
          <tr class="even:bg-indigo-600">
            <td class="whitespace-nowrap pr-5">{row.id}</td>
            <td class="whitespace-nowrap pr-5">{row.last}</td>
            <td class="whitespace-nowrap pr-5">{row.first}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}
