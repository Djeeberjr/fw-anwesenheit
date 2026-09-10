<script lang="ts">
  type LogLevel = "log" | "info" | "warn" | "error" | "debug";

  interface LogEntry {
    level: LogLevel;
    time: string;
    text: string;
  }

  let logs = $state<LogEntry[]>([]);
  const maxLogs = 50;

  function serialize(arg: unknown): string {
    if (typeof arg === "string") return arg;
    if (arg instanceof Error) return arg.stack || arg.message;
    try {
      return JSON.stringify(arg, null, 2);
    } catch {
      return String(arg);
    }
  }

  function push(level: LogLevel, args: unknown[]): void {
    const entry: LogEntry = {
      level,
      time: new Date().toLocaleTimeString(),
      text: args.map(serialize).join(" "),
    };
    logs.push(entry);
    if (logs.length > maxLogs) logs.shift();
  }

  $effect(() => {
    console.debug("Patching console log functions");

    const levels: LogLevel[] = ["log", "info", "warn", "error", "debug"];
    const original = {} as Record<LogLevel, (...args: unknown[]) => void>;

    // Patch console functions
    for (const level of levels) {
      original[level] = console[level].bind(console);
      console[level] = (...args: unknown[]) => {
        push(level, args);
        original[level](...args);
      };
    }

    // Catch uncaught errors and unhandled promise rejections
    const onError = (event: ErrorEvent) => {
      push("error", [
        event.message,
        `(${event.filename}:${event.lineno}:${event.colno})`,
      ]);
    };
    const onRejection = (event: PromiseRejectionEvent) => {
      push("error", ["Unhandled rejection:", event.reason]);
    };

    window.addEventListener("error", onError);
    window.addEventListener("unhandledrejection", onRejection);

    // teardown function
    return () => {
      for (const level of levels) console[level] = original[level];
      window.removeEventListener("error", onError);
      window.removeEventListener("unhandledrejection", onRejection);
    };
  });
</script>

<h2 class="text-xl font-bold">Console</h2>
<ol>
  {#each logs as entry}
    <li class="flex justify-start gap-2">
      <span>{entry.time}</span>
      <span>[{entry.level}]</span>
      <pre>{entry.text}</pre>
    </li>
  {/each}
</ol>
