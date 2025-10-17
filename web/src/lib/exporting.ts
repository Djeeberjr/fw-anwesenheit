import { generateCSVString } from "./csv";
import { dayToDate, fetchDay, fetchDays, type AttendanceDay, type Day } from "./Day";
import type { IDMap } from "./IDMapping";

interface CSVRow {
  ID: string
  Vorname: string
  Nachname: string
  [key: string]: string | boolean
}

function prepareRows(mapping: IDMap, days: AttendanceDay[]): CSVRow[] {
  let csvData: CSVRow[] = [];

  const allIDs = Object.keys(mapping);

  for (const id of allIDs) {
    const name = mapping[id];
    const row: CSVRow = {
      ID: id,
      Vorname: name.first,
      Nachname: name.last,
    };

    for (const day of days) {
      const dayKey = dayToDate(day.date).toLocaleDateString();
      row[dayKey] = day.ids.includes(id);
    }

    csvData.push(row);
  }

  return csvData;
}

async function getDays(from: Date, to: Date): Promise<AttendanceDay[]> {
  const recordedDays: Day[] = await fetchDays(from, to);
  let days: AttendanceDay[] = [];

  for (const day of recordedDays) {
    days.push(await fetchDay(day))
  }

  return days;
}

export async function generateCSVFile(from: Date, to: Date, mapping: IDMap): Promise<string> {
  const days = await getDays(from, to);
  const rows = prepareRows(mapping, days);
  const csvString = generateCSVString(rows);

  return csvString;
}
