<script lang="ts">
  function cleanup(e: SubmitEvent) {
    e.preventDefault();
    console.debug("Cleanup called");
  }
</script>

<div>
  <h2 class="text-xl font-bold">Cleanup</h2>
  <form onsubmit={cleanup}>
    <label class="flex justify-between w-full">
      <span>Von:</span>
      <input type="date" />
    </label>
    <label class="flex justify-between w-full">
      <span>Bis:</span>
      <input type="date" />
    </label>
    <label class="flex justify-between w-full">
      <span>Minimal Anzahl</span>
      <input type="number" defaultValue="1" min="1" class="max-w-20" />
    </label>
    <button
      class="px-6 py-3 text-lg font-semibold text-white bg-indigo-600 rounded-2xl shadow-md hover:bg-indigo-700 transition"
      >Cleanup</button
    >
  </form>
</div>
