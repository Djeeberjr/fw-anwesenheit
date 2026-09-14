import { fetchID, fetchIDList } from "./api"

export interface IDMap {
  [name: string]: Name
}

export interface Name {
  first: string,
  last: string,
}

export async function fetchMapping(): Promise<IDMap> {
  let ids = await fetchIDList();

  let map: IDMap = {};

  for (const id of ids) {
    let id_name = await fetchID(id);
    map[id] = id_name;
  }

  return map;
}

const CACHE_KEY = "idmap";

export function cacheMappingInLocalstore(mapping: IDMap) {
  if (!localStorage) {
    console.error("localStorage is not available");
    return;
  }

  localStorage.setItem(CACHE_KEY, JSON.stringify(mapping));
}

export function loadCachedMappingFromLocalstore(): IDMap | null {
  if (!localStorage) {
    console.error("localStorage is not available");
    return null;
  }

  const data = localStorage.getItem(CACHE_KEY);
  if (!data) {
    return null;
  }

  const mapping = JSON.parse(data);
  return mapping;
}
