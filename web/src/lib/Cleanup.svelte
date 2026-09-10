<script lang="ts">
  import { cleanupPrepare, type DayToBeRemoved } from "./cleanup";
  import { dayToDate, removeDay } from "./Day";

  let fromDate: string | undefined = $state();
  let toDate: string | undefined = $state();
  let ammount: number = $state(1);

  let daysToRemove: DayToBeRemoved[] | undefined = $state();

  let lock = $state(false);

  async function onSubmitShowDays(e: SubmitEvent) {
    e.preventDefault();
    lock = true;

    if (!fromDate || !toDate) {
      lock = false;
      return;
    }

    daysToRemove = await cleanupPrepare(
      new Date(fromDate),
      new Date(toDate),
      ammount,
    );

    lock = false;
  }

  async function onSubmitDelete() {
    lock = true;

    if (daysToRemove == undefined) {
      lock = false;
      return;
    }

    for (const day of daysToRemove) {
      let res = await removeDay(day.day);
      if (res.status >= 400) {
        console.error(res);
      }
    }

    daysToRemove = undefined;
    lock = false;
  }
</script>

<div>
  <h2 class="text-xl font-bold">Cleanup</h2>
  <form onsubmit={onSubmitShowDays}>
    <label class="flex justify-between w-full">
      <span>Von:</span>
      <input type="date" required bind:value={fromDate} />
    </label>
    <label class="flex justify-between w-full">
      <span>Bis:</span>
      <input type="date" required bind:value={toDate} />
    </label>
    <label class="flex justify-between w-full">
      <span>Minimal Anzahl</span>
      <input
        type="number"
        defaultValue="1"
        min="1"
        bind:value={ammount}
        class="max-w-20"
      />
    </label>
    <div class="flex justify-center">
      <button class="default-btn" disabled={lock}>Tage anzeigen</button>
    </div>
  </form>
  {#if daysToRemove != undefined}
    <table class="px-10">
      <thead>
        <tr>
          <th>Datum</th>
          <th>Anwesende</th>
        </tr>
      </thead>

      <tbody>
        {#each daysToRemove as day}
          <tr class="whitespace-nowrap pr-5 pl-2 py-1">
            <td>
              {dayToDate(day.day).toLocaleDateString()}
            </td>
            <td>
              {day.ammount}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    <div class="flex justify-center">
      <button
        class="default-btn !bg-red-600 hover:!bg-red-700 disabled:!bg-gray-600 disabled:hover:!bg-gray-700"
        onclick={onSubmitDelete}
        disabled={lock}>Tage entfernen</button
      >
    </div>
  {/if}
</div>
