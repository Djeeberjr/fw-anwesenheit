<script lang="ts">
  import { onMount } from "svelte";
  import IDTable from "./lib/IDTable.svelte";
  import LastId from "./lib/LastID.svelte";
  import AddIDModal from "./lib/AddIDModal.svelte";
  import ExportModal from "./lib/ExportModal.svelte";
  import { generateCSVFile } from "./lib/exporting";
  import {
    cacheMappingInLocalstore,
    fetchMapping,
    loadCachedMappingFromLocalstore,
    type IDMap,
  } from "./lib/IDMapping";
  import { downloadBlob } from "./lib/downloadBlob";
  import TimeSync from "./lib/TimeSync.svelte";

  let lastID: string = $state("");
  let mapping: IDMap | null = $state(null);

  let addModal: AddIDModal;
  let exportModal: ExportModal;
  let showMaintain = $state(false);

  onMount(async () => {
    mapping = loadCachedMappingFromLocalstore();

    let fetchedMapping = await fetchMapping();
    mapping = fetchedMapping;
    cacheMappingInLocalstore(fetchedMapping);

    let sse = new EventSource("/api/idevent");
    sse.addEventListener("msg", function (e) {
      lastID = e.data;
    });
  });
</script>

<main
  class="bg-gradient-to-br from-blue-100 to-indigo-200 min-h-screen flex flex-col items-center justify-start py-10"
>
  <div class="text-center space-y-6 mb-10">
    <h1 class="text-3xl sm:text-4xl font-bold text-gray-800">Anwesenheit</h1>
  </div>

  <button
    class="px-6 py-3 text-lg font-semibold text-white bg-indigo-600 rounded-2xl shadow-md hover:bg-indigo-700 transition"
    onclick={() => {
      exportModal.open();
    }}
  >
    Export CSV
  </button>

  <div class="pt-3 pb-2">
    <LastId
      id={lastID}
      onAdd={(id) => {
        addModal.open(id);
      }}
    />
  </div>
  <div>
    {#if mapping}
      <IDTable
        data={mapping}
        onEdit={(id, firstName, lastName) => {
          addModal.open(id, firstName, lastName);
        }}
      />
      <span>Gesammmte einträge: {Object.keys(mapping).length}</span>
    {:else}
      Lade ...
    {/if}
  </div>
  <div class="w-118 pt-3">
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
    {/if}
  </div>

  <AddIDModal
    bind:this={addModal}
    onSubmitted={async (id, firstName, lastname) => {
      if (mapping == null) {
        return;
      }

      mapping[id] = {
        first: firstName,
        last: lastname,
      };
    }}
  />

  <ExportModal
    bind:this={exportModal}
    onSubmitted={async (from, to) => {
      if (!mapping) {
        return;
      }
      let csvFile = await generateCSVFile(from, to, mapping);

      downloadBlob("export.csv", csvFile, "text/csv");
    }}
  />
</main>
