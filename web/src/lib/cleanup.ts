import { dayToDate, fetchDay, fetchDays, type Day } from "./Day";

export interface DayToBeRemoved {
  day: Day,
  ammount: number
}

export async function cleanupPrepare(from: Date, to: Date, ammount: number) {
  const recordedDays: Day[] = await fetchDays(from, to);
  let daysToBeRemoved: DayToBeRemoved[] = [];

  for (const day of recordedDays) {
    const fetchedDay = await fetchDay(day);

    if (fetchedDay.ids.length >= ammount) {
      continue
    }

    daysToBeRemoved.push({
      day: fetchedDay.date,
      ammount: fetchedDay.ids.length,
    });
  }

  return daysToBeRemoved;
}
