<script lang="ts">
  import { onMount } from "svelte";
  import IDTable from "./lib/IDTable.svelte";
  import LastId from "./lib/LastID.svelte";

  let lastID: string = $state("");

  onMount(() => {
    let sse = new EventSource("/api/idevent");

    sse.onmessage = (e) => {
      lastID = e.data;
    };
  });
</script>

<main
  class="bg-gradient-to-br from-blue-100 to-indigo-200 min-h-screen flex flex-col items-center justify-start py-10"
>
  <div class="text-center space-y-6 mb-10">
    <h1 class="text-3xl sm:text-4xl font-bold text-gray-800">Anwesenheit</h1>
  </div>

  <a
    class="px-6 py-3 text-lg font-semibold text-white bg-indigo-600 rounded-2xl shadow-md hover:bg-indigo-700 transition"
    href="/api/csv"
    download="anwesenheit.csv"
  >
    Download CSV
  </a>

  <div class="pt-3 pb-2">
    <LastId
      id={lastID}
      onAdd={(id) => {
        console.debug("Add id " + id);
      }}
    />
  </div>
  <div>
    <IDTable />
  </div>
</main>
