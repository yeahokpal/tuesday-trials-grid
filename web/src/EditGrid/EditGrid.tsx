import classNames from "classnames";
import { Database } from "sql.js";
import TrialsGrid from "../TrialsGrid";
import { useEffect, useState } from "react";
import { COLUMNS, ROWS } from "../constants";

import queryFile from '../GridQueries.toml'
import EditLabel from "./EditLabel";
import EditSquare from "./EditSquare";
import { getQueryVars } from "../queryfuncs";

function EditGrid({db, customData, setCustomData}: {db: Database, customData: string | null, setCustomData: (customData: string | null) => void}) {
  const [data, setData] = useState<(QueryData | undefined)[]>(customData? JSON.parse(customData) : [...new Array(ROWS + COLUMNS)].map(() => undefined));
  const [grid, setGrid] = useState<TrialsGrid | undefined>(undefined);

  useEffect(() => {
    setGrid(new TrialsGrid(db, data.map(d => d!), false));
  }, [data]);
  console.log(grid);
  useEffect(() => {
    if (grid?.valid) {
      setCustomData(JSON.stringify(grid.data));
    }
  }, [grid?.valid]);

  function setOneData(i: number, d: QueryData | undefined) {
    let newData = [...data];
    newData[i] = d;
    setData(newData);
  }
  return <>
    <div className={classNames("grid", {"invalid": !(grid?.valid || false)})}>
      {data.slice(ROWS, ROWS + COLUMNS).map((c, i) =>
      <div key={i} style={{gridColumn: i + 2}}>
        <EditLabel data={c} setData={d => setOneData(ROWS + i, d)} />
      </div>)}
      {data.slice(0, ROWS).map((r, i) =>
      <div key={i} style={{gridRow: i + 2}}>
        <EditLabel data={r} setData={d => setOneData(i, d)}/>
      </div>)}
      {(grid?.answers ?? [...new Array(ROWS * COLUMNS).map(() => undefined)]).map((a, i) =>
      <div key={i} style={{gridRow: i / 3 + 2, gridColumn: i % 3 + 2}}>
        <EditSquare answers={a}/>
      </div>)}
    </div>
    <button onClick={() => navigator.clipboard.writeText(JSON.stringify(grid?.data))}>Copy Seed</button>
  </>;
}
export default EditGrid