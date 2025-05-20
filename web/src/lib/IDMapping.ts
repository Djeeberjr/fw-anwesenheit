export interface IDMapping {
  id_map: IDMap
}

export interface IDMap {
  [name: string]: Name
}

export interface Name {
  first: string,
  last: string,
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
