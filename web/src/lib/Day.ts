export type Day = number;

export interface AttendanceDay {
  date: Day,
  ids: string[],
}

export function dayToDate(day: Day): Date {
  const SEC_PER_DAY = 86_400;

  return new Date(day * SEC_PER_DAY * 1000);
}
