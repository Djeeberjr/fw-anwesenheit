<script lang="ts">
  import Modal from "./Modal.svelte";

  let { onSubmitted }: { onSubmitted?: () => void } = $props();

  let displayID = $state("");
  let firstName = $state("");
  let lastName = $state("");

  let modal: Modal;

  export function open(presetID: string, presetFirstName?: string, presetLastName?: string) {
    displayID = presetID;

    firstName = presetFirstName ?? "";
    lastName = presetLastName ?? "";

    modal.open();
  }

  function onsubmit() {
    let data = {
      id: displayID,
      name: {
        first: firstName,
        last: lastName,
      },
    };

    fetch("/api/mapping", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(data),
    }).then(() => {
      onSubmitted?.();
    });

    firstName = "";
    lastName = "";
    displayID = "";
  }
</script>

<Modal bind:this={modal}>
  <form method="dialog" {onsubmit} class="flex flex-col">
    <label class="form-row">
      <span>ID:</span>
      <input type="text" class="form-input" required bind:value={displayID} />
    </label>

    <label class="form-row">
      <span>Vorname:</span>
      <input type="text" class="form-input" required bind:value={firstName} />
    </label>

    <label class="form-row">
      <span>Nachname:</span>
      <input type="text" class="form-input" required bind:value={lastName} />
    </label>

    <div class="flex justify-end mt-3">
      <button
        type="reset"
        class="mr-5 px-2 py-1 bg-red-500 rounded-2xl shadow-md"
        onclick={() => {
          modal.close();

          firstName = "";
          lastName = "";
          displayID = "";
        }}>Abbrechen</button
      >
      <button
        type="submit"
        class="px-2 py-1 bg-indigo-600 rounded-2xl shadow-md hover:bg-indigo-700 transition"
        >Hinzufügen</button
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
    @apply ml-10 border-b-1;
  }
</style>
