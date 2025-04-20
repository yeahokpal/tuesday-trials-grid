import { useEffect, useState } from 'react'
import './App.css'
import initSqlJs, { Database } from "sql.js";

import dbFile from "../../db.sqlite?url";
import sqlWasm from "../node_modules/sql.js/dist/sql-wasm.wasm?url";
import Grid from './Grid/Grid';
import EditGrid from './EditGrid/EditGrid';

function App() {
  const searchParams = new URLSearchParams(window.location.search);
  const customData = searchParams.get("customData");
  const [db, setDb] = useState<Database | null>(null);
  const [edit, setEdit] = useState(false);

  useEffect(() => {
    initSqlJs({locateFile: () => sqlWasm })
      .then(SQL => fetch(dbFile)
        .then(db => db.bytes())
        .then(bytes => setDb(new SQL.Database(bytes))));
  }, []);

  function setCustomData(customData: string | null) {
    if (searchParams.get("customData") != customData)
    {
      searchParams.set("customData", customData ?? "");
      window.location.search = searchParams.toString();
    }
  }

  function loadData() {
    setCustomData(prompt("Enter grid data:"));
  }
  function editMode() {
    setEdit(!edit);
  }

  if (db === null) return <>Loading...</>

  return (
    <>
      <div style={{position: "absolute", right: "0%"}}>
        <button onClick={loadData}>Load Custom Grid</button>
        <br/>
        <button hidden onClick={editMode}>Toggle Edit Mode</button>
      </div>
      <h1>Tuesday Trials Grid</h1>
      {edit ? <EditGrid db={db} customData={customData} setCustomData={setCustomData}/> : <Grid key={customData} db={db} customData={customData}/>}
    </>
  )
}

export default App
