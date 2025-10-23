<script lang="ts">
  import Modal from "./Modal.svelte";

  let { onSubmitted }: { onSubmitted?: (from: Date, to: Date) => void } =
    $props();

  let modal: Modal;

  let fromDate: string | undefined = $state();
  let toDate: string | undefined = $state();

  let selectedYear: number = $state(new Date().getFullYear());

  let selectedTab = $state(0);

  export function open() {
    modal.open();
  }

  function generateYears() {
    const currentYear = new Date().getFullYear();
    const startingYear = 2020;

    return Array.from(
      new Array(currentYear + 1 - startingYear),
      (_, i) => i + startingYear,
    );
  }

  function onsubmit(e: SubmitEvent) {
    let from: Date;
    let to: Date;

    switch (selectedTab) {
      case 0:
        if (!fromDate || !toDate) {
          e.preventDefault();
          return;
        }

        from = new Date(fromDate);
        to = new Date(toDate);
        break;
      case 1:
        from = new Date(selectedYear, 0);
        to = new Date(selectedYear + 1, 0);
        break;
      default:
        console.error("Invalid tab");
        return;
    }

    onSubmitted?.(from, to);
  }
</script>

<Modal bind:this={modal}>
  <div class="flex">
    <button
      onclick={() => {
        selectedTab = 0;
      }}
      class="tab {selectedTab === 0 ? 'tab-active' : ''}"
    >
      Datum
    </button>
    <button
      onclick={() => {
        selectedTab = 1;
      }}
      class="tab {selectedTab === 1 ? 'tab-active' : ''}"
    >
      Jahr
    </button>
  </div>
  <form method="dialog" {onsubmit} class="flex flex-col">
    {#if selectedTab === 0}
      <div>
        <label class="form-row">
          <span>Von:</span>
          <input type="date" class="form-input" bind:value={fromDate} />
        </label>

        <label class="form-row">
          <span>Bis:</span>
          <input type="date" class="form-input" bind:value={toDate} />
        </label>
      </div>
    {/if}

    {#if selectedTab === 1}
      <div>
        <label class="form-row">
          <span>Kalendar Jahr:</span>
          <select class="form-input" bind:value={selectedYear}>
            {#each generateYears() as year}
              <option value={year}>{year}</option>
            {/each}
          </select>
        </label>
      </div>
    {/if}

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

  .tab {
    @apply px-4 py-2 rounded-t-lg bg-indigo-600 hover:bg-indigo-700 font-medium border-b-2 border-transparent cursor-pointer transition-colors duration-200;
  }

  .tab-active {
    @apply px-4 py-2 bg-indigo-500 font-semibold border-b-2 border-blue-600 shadow-sm cursor-pointer;
  }

  .form-row {
    @apply flex justify-between my-1;
  }

  .form-input {
    @apply ml-20;
  }
</style>
