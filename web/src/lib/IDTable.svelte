<script lang="ts">
  import { onMount } from "svelte";
  import { fetchMapping, type IDMap } from "./IDMapping";
  let data: IDMap | undefined = $state();

  let {
    onEdit,
  }: { onEdit?: (id: string, firstName: string, lastName: string) => void } =
    $props();

  export async function reloadData() {
    data = await fetchMapping();
  }

  let rows = $derived(
    data
      ? Object.entries(data).map(([id, value]) => ({
          id,
          ...value,
        }))
      : [],
  );

  let sortKey: keyof (typeof rows)[0] = $state("last");
  let sortDirection: "asc" | "desc" = $state("asc");

  let rowsSorted = $derived(
    [...rows].sort((a, b) => {
      let cmp = String(a[sortKey]).localeCompare(String(b[sortKey]));
      return sortDirection === "asc" ? cmp : -cmp;
    }),
  );

  function handleSortClick(key: keyof (typeof rows)[0]) {
    if (sortKey === key) {
      sortDirection = sortDirection === "asc" ? "desc" : "asc";
    } else {
      sortKey = key;
      sortDirection = "asc";
    }
  }

  function indicator(key: keyof (typeof rows)[0]) {
    if (sortKey !== key) return "";
    return sortDirection === "asc" ? "▲" : "▼";
  }

  onMount(async () => {
    await reloadData();
  });
</script>

{#if data == null}
  Loading...
{:else}
  <div class="bg-indigo-500 py-2 rounded-2xl overflow-x-auto">
    <table class="px-10">
      <thead>
        <tr>
          <th
            class="text-left pr-5 pl-2 cursor-pointer select-none"
            onclick={() => {
              handleSortClick("id");
            }}
          >
            ID
            <span class="indicator">{indicator("id")}</span>
          </th>
          <th
            class="text-left pr-5 cursor-pointer select-none"
            onclick={() => {
              handleSortClick("last");
            }}
          >
            Nachname
            <span class="indicator">{indicator("last")}</span>
          </th>
          <th
            class="text-left pr-5 cursor-pointer select-none"
            onclick={() => {
              handleSortClick("first");
            }}
            >Vorname

            <span class="indicator">{indicator("first")}</span>
          </th>
          <th> </th>
        </tr>
      </thead>
      <tbody>
        {#each rowsSorted as row}
          <tr class="even:bg-indigo-600">
            <td class="whitespace-nowrap pr-5 pl-2 py-1">{row.id}</td>
            <td class="whitespace-nowrap pr-5">{row.last}</td>
            <td class="whitespace-nowrap pr-5">{row.first}</td>
            <td class="pr-5"
              ><button
                onclick={() => {
                  onEdit && onEdit(row.id, row.first, row.last);
                }}
                class="cursor-pointer">🔧</button
              ></td
            >
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}

<style lang="css" scoped>
  @reference "../app.css";
  .indicator {
    @apply ml-1 w-4 inline-block;
  }
</style>
