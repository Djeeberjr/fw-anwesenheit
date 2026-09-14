import type { AttendanceDay, Day } from "./Day";
import type { Name } from "./IDMapping";

/**
  * Get the attendance for a single day
  */
export async function fetchDay(day: Day): Promise<AttendanceDay> {
  const q = (new URLSearchParams({ day: day.toString() }).toString());
  let res = await fetch("/api/day?" + q);

  if (res.status >= 400) {
    const errorMessage = `Failed to fetch day: ${day} with status ${res.status}: ${await res.text()}`;
    console.error(errorMessage);
    throw new Error(errorMessage);
  }

  return await res.json();
}

/**
  * Get all the dates in a timespan where attendance was recored
*/
export async function fetchDays(from: Date, to: Date): Promise<Day[]> {
  const q = new URLSearchParams({ from: (from.getTime() / 1000).toString(), to: (to.getTime() / 1000).toString() });
  let res = await fetch("/api/days?" + q);

  if (res.status >= 400) {
    const errorMessage = `Failed to fetch days: from ${from} to ${to} with status ${res.status}: ${await res.text()}`;
    console.error(errorMessage);
    throw new Error(errorMessage);
  }

  return await res.json();
}

/**
  * Remove a single day
*/
export async function removeDay(day: Day): Promise<void> {
  let res = await fetch("/api/day?" + (new URLSearchParams({ day: day.toString() }).toString()), {
    method: "DELETE",
  })

  if (res.status >= 400) {
    const errorMessage = `Failed to remove day: ${day} with status ${res.status}: ${await res.text()}`;
    console.error(errorMessage);
    throw new Error(errorMessage);
  }
}

/**
  * Add or update a name for an ID
*/
export async function addMapping(firstName: string, lastName: string, id: string): Promise<void> {
  const payload = {
    id: id,
    name: {
      first: firstName,
      last: lastName,
    },
  };

  let res = await fetch("/api/mapping", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  })

  if (res.status >= 400) {
    const errorMessage = `Failed to add mapping: ${id} ${firstName} ${lastName} with status ${res.status}: ${await res.text()}`;
    console.error(errorMessage);
    throw new Error(errorMessage);
  }

}

/**
  * Get all ids where there exist a mapping to
*/
export async function fetchIDList(): Promise<string[]> {
  let res = await fetch("/api/mappings");

  if (res.status >= 400) {
    const errorMessage = `Failed to fetch id list with status ${res.status}: ${await res.text()}`;
    console.error(errorMessage);
    throw new Error(errorMessage);
  }

  return await res.json();
}

/**
  * Get the name for a single id
*/
export async function fetchID(id: string): Promise<Name> {
  let res = await fetch("/api/mapping?id=" + id);

  if (res.status >= 400) {
    const errorMessage = `Failed to fetch id name: ${id} with status ${res.status}: ${await res.text()}`;
    console.error(errorMessage);
    throw new Error(errorMessage);
  }

  return await res.json();
}

export async function getRTCTime(): Promise<Date> {
  let res = await fetch("/api/time");

  if (res.status >= 400) {
    const errorMessage = `Failed to get the RTC time with status ${res.status}: ${await res.text()}`;
    console.error(errorMessage);
    throw new Error(errorMessage);
  }

  let timestamp = parseInt(await res.text());
  return new Date(timestamp * 1000);
}

/**
  * Set the time for the internal RTC 
*/
export async function setRTCTime(time: Date): Promise<void> {
  let res = await fetch("/api/time", {
    method: "POST",
    headers: {
      "Content-type": "application/json; charset=UTF-8",
    },
    body: JSON.stringify(time.getTime()),
  });

  if (res.status >= 400) {
    const errorMessage = `Failed to set the RTC time to: ${time} with status ${res.status}: ${await res.text()}`;
    console.error(errorMessage);
    throw new Error(errorMessage);
  }
}

