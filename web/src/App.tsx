import { useEffect, useState } from 'react'
import './App.css'
import initSqlJs, { Database } from "sql.js";

import dbFile from "../../db.sqlite?url";
import sqlWasm from "../node_modules/sql.js/dist/sql-wasm.wasm?url";
import Grid from './Grid/Grid';
import EditGrid from './EditGrid/EditGrid';
import { useVersusConnection } from './VersusGrid/useVersusConnection';
import VersusGrid from './VersusGrid/VersusGrid';

function App() {
  const searchParams = new URLSearchParams(window.location.search);
  const customData = searchParams.get("customData");
  const [db, setDb] = useState<Database | null>(null);
  const [edit, setEdit] = useState(false);
  const versus = useVersusConnection(db);

  useEffect(() => {
    initSqlJs({locateFile: () => sqlWasm })
      .then(SQL => fetch(dbFile)
        .then(db => db.bytes())
        .then(bytes => {
          let db = new SQL.Database(bytes);
          setDb(db);
          // console.debug(JSON.stringify(
          //   Object.fromEntries(["4/20/2025", "4/21/2025", "4/22/2025", "4/23/2025"].map(d => [d, TrialsGrid.getRandomValidGrid(db, d).data]))
          // ));
    }));
  }, []);

  useEffect(() => {
    if (searchParams.has("host") && searchParams.has("id")) {
      versus.setHost(searchParams.get("host") === "true");
      versus.setId(searchParams.get("id"));
    }
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
      {edit && <EditGrid db={db} customData={customData} setCustomData={setCustomData}/>}
      {versus.connected && <VersusGrid db={db} connection={versus}/>}
      {!edit && !versus.connected && <Grid key={customData} db={db} customData={customData}/>}
    </>
  )
}

export default App
