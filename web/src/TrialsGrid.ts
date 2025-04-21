import prand from "pure-rand";
import { RandomGenerator } from "pure-rand/types/RandomGenerator";
import { Database } from "sql.js";
import queryFile from "./GridQueries.toml";
import { COLUMNS, ROWS } from "./constants";

import 'core-js/actual';
import { toQuery, queryDataEquals, getQueryVars } from "./queryfuncs";

const cyrb53 = (str: string, seed : number = 0) => {
    let h1 = 0xdeadbeef ^ seed, h2 = 0x41c6ce57 ^ seed;
    for(let i = 0, ch; i < str.length; i++) {
        ch = str.charCodeAt(i);
        h1 = Math.imul(h1 ^ ch, 2654435761);
        h2 = Math.imul(h2 ^ ch, 1597334677);
    }
    h1  = Math.imul(h1 ^ (h1 >>> 16), 2246822507);
    h1 ^= Math.imul(h2 ^ (h2 >>> 13), 3266489909);
    h2  = Math.imul(h2 ^ (h2 >>> 16), 2246822507);
    h2 ^= Math.imul(h1 ^ (h1 >>> 13), 3266489909);
  
    return 4294967296 * (2097151 & h2) + (h1 >>> 0);
};

function getOrInitValues(sql: Database, v: Var) {
    return v.values ??= sql.exec(v.query!)[0].values.flat().map(va => va!.toString());
}
class TrialsGrid {
    rows: (Query | undefined)[];
    columns: (Query | undefined)[];
    answers: (string[] | undefined)[];
    data: QueryData[];
    sql: Database;
    valid: boolean;

    public constructor(sql: Database, data: QueryData[], shortCircuit: boolean) {
        Object.values(queryFile.vars).forEach(v => getOrInitValues(sql, v));
        this.sql = sql;
        this.data = data;
        let loaded = data.map(d => d? toQuery(queryFile, d) : undefined);
        this.rows = loaded.slice(0, ROWS);
        this.columns = loaded.slice(ROWS, ROWS + COLUMNS);
        this.answers = [...Array(ROWS * COLUMNS)].fill(undefined);
        if (loaded.some(l => !l) && shortCircuit) {
            this.valid = false;
        } else {
            this.calcAnswers(shortCircuit);
            this.valid = this.verifyGrid();
        }
        console.log(this);
    }

    public setRow(i: number, data: QueryData) {
        if (i < 0 || i > ROWS) return;
        if (queryDataEquals(data, this.data[i])) return;
        this.data[i] = data;
        this.rows[i] = toQuery(queryFile, data);
        this.calcAnswers(false, {rows: [i]});
        this.valid = this.verifyGrid();
    }

    public setColumn(i: number, data: QueryData) {
        if (i < 0 || i > COLUMNS) return;
        if (queryDataEquals(data, this.data[ROWS + i])) return;
        this.data[i] = data;
        this.columns[i] = toQuery(queryFile, data);
        this.calcAnswers(false, {columns: [i]});
        this.valid = this.verifyGrid();
    }

    static getRandomGrids = function* (rand: RandomGenerator, limit: number) {
        let queryDup: number[] = [];
        let gameDup: string[] = [];
        const max = queryFile.queries.map(q => q.odds).reduce((a, b) => a + b);
        for (let count = 0; count < limit; count++) {
            let i;
            do {
                let val = prand.unsafeUniformIntDistribution(0, max - 1, rand);
                i = 0;
                for (;i < queryFile.queries.length && val - queryFile.queries[i].odds > 0; i++) {
                    val -= queryFile.queries[i].odds;
                }
            } while (queryFile.queries[i].odds <= 50 && queryDup.includes(i));
            const query = queryFile.queries[i];
            queryDup.push(i);

            yield {i, v: query.options?.at(prand.unsafeUniformIntDistribution(0, query.options.length - 1, rand))
                ?? Object.fromEntries(
                    getQueryVars(queryFile, i)
                    .map(([key, value]) => {
                        const vValues = value.values?.filter(val => key !== "game" && key !== "stream_game" || !gameDup.includes(val));
                        const r = prand.unsafeUniformIntDistribution(0, (vValues?.length ?? 1) - 1, rand);
                        const res = vValues?.at(r) ?? "";
                        if ( key === "game" || key === "stream_game") {
                            gameDup.push(res);
                        }
                        return [key, res];
                    }))
            };
        }
    }

    static getRandomValidGrid(sql: Database, seed: string): TrialsGrid {
        Object.values(queryFile.vars).forEach(v => getOrInitValues(sql, v));
        let rand = prand.xoroshiro128plus(cyrb53(seed));
        let badqueries = 0;
        let grid;
        do {
            grid = new TrialsGrid(sql, Array.from(TrialsGrid.getRandomGrids(rand, ROWS + COLUMNS)), true);
        } while (!grid.valid && ++badqueries);
        console.log("Generated %d grids", badqueries);

        return grid;
    }

    private calcAnswers(shortCircuit: boolean, {rows, columns}: {rows?: number[], columns?: number[]} = {}) {
        rows ??= this.rows.map((_, i) => i);
        columns ??= this.columns.map((_, i) => i);

        for (let [row, col] of rows.flatMap(r => columns.map((c): [row: number, col: number] => [r, c]))) {
            let i = row * COLUMNS + col;
            if (this.rows[row] && this.columns[col]) {
                this.answers[i] = this.sql.exec(`${this.rows[row].query} INTERSECT ${this.columns[col].query}`)
                                .map(r => r.values).flat().map(res => res!.toString());
            }
            if (shortCircuit && (this.answers[i]?.length ?? 0) == 0) return false;
        }
    }

    private verifyGrid() {
        const residualPath = (i: number, graph: Set<number>[], visited: boolean[]): number => {
            if (i === graph.length - 1) return 1;

            for(let v of graph[i]) {
                if (!visited[v]) {
                    visited[v] = true;
                    graph[i].delete(v);
                    graph[v].add(i);
                    if (residualPath(v, graph, visited) > 0) return 1;
                    graph[v].delete(i);
                    graph[i].add(v);
                }
            }
            return 0;
        };
        if (this.answers.length !== ROWS * COLUMNS || this.answers.some(a => !a || a.length === 0)) return false;
        let indices = new Map(([...new Set(this.answers.flat())].map((v, i) => [v, i])));
        
        let graph: Set<number>[] = Array.from({length: ROWS * COLUMNS + indices.size + 1}, () => new Set());
        this.answers.forEach((arr, i) => arr!.forEach(v => graph[i].add(ROWS * COLUMNS + indices.get(v)!)));
        indices.forEach(i => graph[ROWS * COLUMNS + i].add(ROWS * COLUMNS + indices.size));
        return this.answers.every((_, i) => residualPath(i, graph, []));
    }
}
export default TrialsGrid