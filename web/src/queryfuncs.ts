export function getQueryVars(queryFile: QueryFile, q: number) {
    return Object.entries(queryFile.vars)
        .filter(([k, _]) => queryFile.queries[q].query.includes(`[${k}]`));
}

export function queryDataEquals(a: QueryData, b: QueryData) {
    if (a.i !== b.i) return false;
    let ak = Object.keys(a.v);
    let bk = Object.keys(b.v);
    if (ak.length != bk.length) return false;
    return ak.every(k => a.v[k] === b.v[k]);
}
export function tryToQuery(queryFile: QueryFile, d: QueryData) {
    let q = queryFile.queries.at(d.i)
    if (!q || getQueryVars(queryFile, d.i).some(([k, _]) => !d.v[k])) return;
    return {...q,
        query: Object.entries(d.v).reduce((str, [k, v]) => str.replace(`[${k}]`, v), q.query),
        label: Object.entries(d.v).reduce((str, [k, v]) => str.replace(`[${k}]`, queryFile.vars[k]?.labels?.at(queryFile.vars[k].values!.indexOf(v)) ?? v), q.label),
    };
}
export function toQuery(queryFile: QueryFile, d: QueryData) {
    return tryToQuery(queryFile, d)!;
}