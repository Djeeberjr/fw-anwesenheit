export type Day = number;

export interface AttendanceDay {
  date: Day,
  ids: string[],
}

export function dayToDate(day: Day): Date {
  const SEC_PER_DAY = 86_400;

  return new Date(day * SEC_PER_DAY * 1000);
}

export async function fetchDay(day: Day): Promise<AttendanceDay> {
  let res = await fetch("/api/day?" + (new URLSearchParams({ day: day.toString() }).toString()));

  let json = await res.json();

  return json;
}

export async function fetchDays(from: Date, to: Date): Promise<Day[]> {
  let q = new URLSearchParams({ from: (from.getTime() / 1000).toString(), to: (to.getTime() / 1000).toString() });

  let res = await fetch("/api/days?" + q);

  let json = await res.json();

  return json;
}

