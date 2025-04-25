import { Database } from "sql.js";
import TrialsGrid from "./TrialsGrid";

export function getQueryVars(queryFile: QueryFile, query: Query) {
    return Object.entries(queryFile.vars)
        .filter(([k, _]) => query.query.includes(`[${k}.`) || query.query.includes(`[${k}]`));
}

export function queryDataEquals(a: QueryData, b: QueryData) {
    if (a.id !== b.id) return false;
    let ak = Object.keys(a.v);
    let bk = Object.keys(b.v);
    if (ak.length != bk.length) return false;
    return ak.every(k => a.v[k] === b.v[k]);
}
export function toQuery(queryFile: QueryFile, d: QueryData) {
    let q = queryFile.queries[d.id];
    if (!q) return;

    let varStrings = Object.entries(d.v)
        .map(([k, v]) => ({k, v: queryFile.vars[k].values?.find(val => val['id'] === v) ?? v}))
        .flatMap(({k, v}) => typeof v === 'string'? [[k, v]] : Object.entries(v).map(([k2, v2]) => [k2 === "id"? k : k+"."+k2, v2]));

    return {...q,
        query: varStrings.reduce((str, [k, v]) => str.replace(`[${k}]`, v), q.query),
        label: varStrings.reduce((str, [k, v]) => str.replace(`[${k}]`, v), q.label),
    };
}
export function getCustomGrid(db: Database, customData: string | null) {
  let newData: QueryData[] = JSON.parse(customData ?? "null");
  if (newData) {
    let grid = new TrialsGrid(db, newData, true);
    if (grid.valid) return grid;
  }
}