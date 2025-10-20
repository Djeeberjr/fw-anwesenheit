interface CSVOptions {
  delimiter?: string;
  headerOrder?: string[];
  eol?: string;
  includeBOM?: boolean;
  nullString?: string;
}

type RowObject = Record<string, any>;
type InputRows = RowObject[];

export function generateCSVString(input: InputRows, opts: CSVOptions = {}): string {
  const {
    delimiter = ",",
    headerOrder,
    eol = "\r\n",
    includeBOM = false,
    nullString = "",
  } = opts;

  // Check if the value need quoting
  // Usually not needed in our use case but still in case
  const needsQuoting = (s: string) =>
    s.includes(delimiter) || s.includes('"') || s.includes("\n") || s.includes("\r") || /^\s|\s$/.test(s);

  // Transform the value of a cell in a SAFE string
  const escapeCell = (raw: any): string => {
    if (raw === null || raw === undefined) return nullString;

    let s = stringify(raw);

    // Replace quotes 
    if (s.includes('"')) s = s.replace(/"/g, '""');

    return needsQuoting(s) ? `"${s}"` : s;
  };

  // Transform the value of a cell into a string
  function stringify(v: any): string {
    if (v === null || v === undefined) return nullString;
    if (v instanceof Date) return v.toLocaleDateString();
    if (typeof v === "boolean") return v ? "X" : "";

    if (typeof v === "object") {
      // CHeck if array and join with "|"
      if (Array.isArray(v)) return v.map(item => (item === null || item === undefined ? nullString : String(item))).join("|");

      // If all fails parse it via json
      // Should also not happen in our use case
      try { return JSON.stringify(v); } catch { return String(v); }
    }
    return String(v);
  }

  const objs = (input as RowObject[]) || [];

  // Derive headers in the order keys are first encountered (fixed: no reduce)
  const seen = new Set<string>();
  const derivedOrder: string[] = [];
  for (const obj of objs) {
    if (!obj || typeof obj !== "object") continue;
    for (const k of Object.keys(obj)) {
      if (!seen.has(k)) {
        seen.add(k);
        derivedOrder.push(k);
      }
    }
  }

  // Apply headerOrder if provided: put listed columns first (in the order provided),
  // then append the remaining derived headers in their derived order.
  let finalHeaders = derivedOrder.slice();
  if (Array.isArray(headerOrder) && headerOrder.length > 0) {
    const headSet = new Set(headerOrder);
    const first = headerOrder.filter(h => seen.has(h)); // keep only headers that actually exist
    const rest = derivedOrder.filter(h => !headSet.has(h));
    finalHeaders = first.concat(rest);
  }

  const rowsOut: string[] = [];

  if (finalHeaders.length > 0) {
    rowsOut.push(finalHeaders.map(h => escapeCell(h)).join(delimiter));
  }

  // Parse every row 
  for (const obj of objs) {

    // Check if obj is null or somthing else we can't convert
    if (!obj || typeof obj !== "object") {

      // produce empty row with same number of columns 
      rowsOut.push(finalHeaders.map(() => escapeCell(null)).join(delimiter));
      continue;
    }

    // For every header check if a value on our row exist and print it
    const row = finalHeaders.map(col => escapeCell(col in obj ? (obj as any)[col] : null)).join(delimiter);
    rowsOut.push(row);
  }

  return (includeBOM ? "\uFEFF" : "") + rowsOut.join(eol);
}
