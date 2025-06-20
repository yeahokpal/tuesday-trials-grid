import {assert, describe, test} from 'vitest';
import queryFile from './GridQueries.toml';
import TrialsGrid from './TrialsGrid';
import { getQueryVars, toQuery } from './queryfuncs';
import initSqlJs from 'sql.js';
import fs from 'fs';
const SQL = await initSqlJs();
const sql = new SQL.Database(fs.readFileSync("../db.sqlite"));
describe("vars", () =>
{
    for (let [key, value] of Object.entries(queryFile.vars))
    {
        test("var "+key, () =>
        {
            assert.doesNotThrow(() => TrialsGrid.initValues(sql, value));
        });
    }
});

describe("queries", () =>
{
    Object.values(queryFile.vars).forEach(v => TrialsGrid.initValues(sql, v));
    for(let [key, value] of Object.entries(queryFile.queries))
    {
        test("query "+key, () =>
        {
            let q = toQuery(queryFile, {
                id: key,
                v: Object.fromEntries(
                    getQueryVars(queryFile, value).map(v =>[v[0], (v[1].strValues?.[0] ?? v[1].values![0].id)])
                )
            })
            assert.isDefined(q);
            assert.doesNotThrow(() => sql.exec(q.query));
        });
    }
});