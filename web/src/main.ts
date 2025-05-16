import './style.css'
import { IDMap, Name, requestMappings } from './idMapping'

function createTableRow(a: string, b: string, c: string): HTMLTableRowElement {
  const tr = document.createElement('tr');
  [a, b, c].forEach(value => {
    const td = document.createElement('td');
    td.textContent = value;
    tr.appendChild(td);
  });
  return tr;
}

function orderByFirstName(obj: IDMap): { id: string, name: Name }[] {
  return Object.entries(obj)
    .sort(([, a], [, b]) => a.first.localeCompare(b.first))
    .map(([id, { first, last }]) => ({ id, name: { first, last } }));
}

requestMappings().then(r => {
  orderByFirstName(r.id_map).forEach(e => {
    let row = createTableRow(e.id, e.name.last, e.name.first);
    document.querySelector("#mappingTable")?.appendChild(row);
  })
})

