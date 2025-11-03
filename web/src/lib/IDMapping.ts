export interface IDMap {
  [name: string]: Name
}

export interface Name {
  first: string,
  last: string,
}

async function fetchIDList(): Promise<string[]> {
  let res = await fetch("/api/mappings");
  let data = await res.json();
  return data;
}

async function fetchID(id: string): Promise<Name> {
  let res = await fetch("/api/mapping?id=" + id);
  let data = await res.json();
  return data;
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

export async function addMapping(id: string, firstName: string, lastName: string) {
  let req = await fetch("/api/mapping", {
    method: "POST",
    headers: {
      "Content-type": "application/json; charset=UTF-8"
    },
    body: JSON.stringify({
      id,
      name: {
        first: firstName,
        lastName: lastName,
      },
    })
  });

  if (req.status != 200) {
    console.error(await req.text())
  }
}
