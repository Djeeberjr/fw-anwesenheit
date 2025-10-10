export interface IDMap {
  [name: string]: Name
}

export interface Name {
  first: string,
  last: string,
}

function stupidSerdeFix(pairs: [string, Name][]): IDMap {
  const map: IDMap = {};
  for (const [key, value] of pairs) {
    map[key] = value;
  }

  return map;
}

export async function fetchMapping(): Promise<IDMap> {
  let res = await fetch("/api/mapping");

  let data = await res.json();

  return stupidSerdeFix(data);
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
