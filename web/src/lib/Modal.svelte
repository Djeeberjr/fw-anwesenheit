<script lang="ts">
  import type { Snippet } from 'svelte';
  let { children }: { children: Snippet } = $props();

  let dialog:  HTMLDialogElement; 

  export function open(){
    dialog.showModal()
  }

  export function close(){
    dialog.close();
  }

</script>

<dialog
  bind:this={dialog}
  closedby="any"
  class="bg-gradient-to-br from-blue-100 to-indigo-200 p-5 center-dialog rounded-2xl backdrop:bg-black/50 backdrop:backdrop-blur-xs"
>
  {@render children?.()}
</dialog>

<style scoped>
  .center-dialog {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
  }

</style>
