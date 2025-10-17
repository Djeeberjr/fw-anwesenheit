<script lang="ts">
  import Modal from "./Modal.svelte";

  let { onSubmitted }: { onSubmitted?: (from: Date, to: Date) => void } = $props();

  let modal: Modal;

  let fromDate: string | undefined = $state();
  let toDate: string | undefined = $state();

  export function open() {
    modal.open();
  }

  function onsubmit(e: SubmitEvent) {
    if (!fromDate || !toDate){
      e.preventDefault();
      return;
    }
    
    let from = new Date(fromDate);
    let to = new Date(toDate);

    onSubmitted?.(from,to);
  }
</script>

<Modal bind:this={modal}>
  <form method="dialog" {onsubmit} class="flex flex-col">
    <label class="form-row">
      <span>Von:</span>
      <input type="date" class="form-input" bind:value={fromDate} />
    </label>

    <label class="form-row">
      <span>Bis:</span>
      <input type="date" class="form-input" bind:value={toDate} />
    </label>

    <div class="flex justify-end mt-3">
      <button
        type="reset"
        class="mr-5 px-2 py-1 bg-red-500 rounded-2xl shadow-md"
        onclick={() => {
          modal.close();

          fromDate = undefined;
          toDate = undefined;
        }}>Abbrechen</button
      >
      <button
        type="submit"
        class="px-2 py-1 bg-indigo-600 rounded-2xl shadow-md hover:bg-indigo-700 transition"
        >Export CSV</button
      >
    </div>
  </form>
</Modal>

<style scoped>
  @reference "../app.css";

  .form-row {
    @apply flex justify-between;
  }

  .form-input {
    @apply ml-10;
  }
</style>
