<script lang="ts">
  import { onMount } from "svelte";
  import IDTable from "./components/IDTable.svelte";
  import LastId from "./components/LastID.svelte";
  import AddIDModal from "./components/AddIDModal.svelte";
  import ExportModal from "./components/ExportModal.svelte";
  import { generateCSVFile } from "./lib/exporting";
  import {
    cacheMappingInLocalstore,
    fetchMapping,
    loadCachedMappingFromLocalstore,
    type IDMap,
  } from "./lib/IDMapping";
  import { downloadBlob } from "./lib/downloadBlob";
  import Maintenance from "./components/Maintenance.svelte";

  let mapping: IDMap | null = $state(null);

  let addModal: AddIDModal;
  let exportModal: ExportModal;

  onMount(async () => {
    mapping = loadCachedMappingFromLocalstore();

    let fetchedMapping = await fetchMapping();
    mapping = fetchedMapping;
    cacheMappingInLocalstore(fetchedMapping);
  });
</script>

<main
  class="bg-gradient-to-br from-blue-100 to-indigo-200 min-h-screen flex flex-col items-center justify-start py-10"
>
  <div class="text-center space-y-6 mb-10">
    <h1 class="text-3xl sm:text-4xl font-bold text-gray-800">Anwesenheit</h1>
  </div>

  <button
    class="default-btn"
    onclick={() => {
      exportModal.open();
    }}
  >
    Export CSV
  </button>

  <section class="pt-3 pb-2">
    <LastId
      onAdd={(id) => {
        addModal.open(id);
      }}
    />
  </section>

  <section>
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
  </section>

  <section class="w-118 pt-3">
    <Maintenance />
  </section>
</main>

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
