<script lang="ts">
  import { onMount } from "svelte";

  let serverTime: Date | null = null;
  let clientTime: Date | null = null;
  let diff = 0;

  onMount(async () => {
    let res = await fetch("/api/time");
    if (!res.ok) {
      console.error("Failed to get time from server: ", res);
      return;
    }

    let timestamp = parseInt(await res.text());
    clientTime = new Date();
    serverTime = new Date(timestamp * 1000);

    diff = clientTime.getTime() - serverTime.getTime();
  });

  function formatDiff(ms: number) {
    if (ms === null) return "--";
    const sign = ms >= 0 ? "+" : "-";
    const abs = Math.abs(ms);

    const seconds = abs / 1000;
    const minutes = seconds / 60;
    const hours = minutes / 60;
    const days = hours / 24;
    const weeks = days / 7;

    let value;
    if (weeks >= 1)
      value = `${weeks.toFixed(1)} Wochen${weeks >= 2 ? "s" : ""}`;
    else if (days >= 1)
      value = `${days.toFixed(1)} Tage${days >= 2 ? "s" : ""}`;
    else if (hours >= 1)
      value = `${hours.toFixed(1)} Stunden${hours >= 2 ? "s" : ""}`;
    else if (minutes >= 1)
      value = `${minutes.toFixed(1)} Minuten${minutes >= 2 ? "s" : ""}`;
    else
      value = `${seconds.toFixed(1)} Sekunden${seconds >= 2 || seconds === 0 ? "s" : ""}`;

    return `${sign}${value}`;
  }
</script>

<h2 class="text-xl font-bold">RTC</h2>
{#if serverTime && clientTime}
  <div class="w-full">
    <div class="flex justify-between w-full">
      <span>RTC:</span>
      <span>{serverTime.toLocaleString()}</span>
    </div>
    <div class="flex justify-between w-full">
      <span>Lokal:</span>
      <span>{clientTime.toLocaleString()}</span>
    </div>
    <div class="flex justify-between w-full">
      <span>Diff: </span>
      <span
        class={Math.abs(diff) > 5 * 60 * 1000
          ? "text-red-600"
          : "text-green-600"}>{formatDiff(diff)}</span
      >
    </div>
    <button
      class="px-6 py-3 text-lg font-semibold text-white bg-indigo-600 rounded-2xl shadow-md hover:bg-indigo-700 transition"
      >Sync now!</button
    >
  </div>
{:else}
  Lade ...
{/if}
