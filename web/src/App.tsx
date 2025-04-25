import { useEffect, useRef, useState } from "react";
import "./App.css";
import initSqlJs, { Database } from "sql.js";

import dbFile from "../../db.sqlite?url";
import sqlWasm from "../node_modules/sql.js/dist/sql-wasm.wasm?url";
import Grid from "./Grid/Grid";
import EditGrid from "./EditGrid/EditGrid";
import Report from "./Report/Report"

function App() {
  const searchParams = new URLSearchParams(window.location.search);
  const customData = searchParams.get("customData");
  const [db, setDb] = useState<Database | null>(null);
  const [edit, setEdit] = useState(false);

  useEffect(() => {
    initSqlJs({ locateFile: () => sqlWasm }).then((SQL) =>
      fetch(dbFile)
        .then((db) => db.bytes())
        .then((bytes) => {
          let db = new SQL.Database(bytes);
          setDb(db);
          // console.debug(JSON.stringify(
          //   Object.fromEntries(["4/20/2025", "4/21/2025", "4/22/2025", "4/23/2025"].map(d => [d, TrialsGrid.getRandomValidGrid(db, d).data]))
          // ));
        })
    );
  }, []);

  function setCustomData(customData: string | null) {
    if (searchParams.get("customData") != customData) {
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
  function showReport() {
    
  }

  const [sideNavOpen, setSideNavOpen] = useState(false);
  const sideNavRef = useRef<HTMLDivElement>(null);

  const toggleSideNav = () => {
    setSideNavOpen((prev) => !prev);
  };

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        sideNavRef.current &&
        !sideNavRef.current.contains(event.target as Node)
      ) {
        setSideNavOpen(false);
      }
    };

    if (sideNavOpen) {
      document.addEventListener("mousedown", handleClickOutside);
    } else {
      document.removeEventListener("mousedown", handleClickOutside);
    }

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [sideNavOpen]);

  if (db === null) return <>Loading...</>;

  return (
    <>
      <div className="navButton" onClick={toggleSideNav}>
        &#9776;
      </div>
      <div
        id="sideNav"
        ref={sideNavRef}
        className={sideNavOpen ? "open" : ""}
      >
        <div className="sideNavItems">
          <button onClick={loadData}>Load Custom Grid</button>
          <button onClick={showReport}>Report Issue</button>
          {/* <button onClick={editMode}>Toggle Edit Mode</button> */}
        </div>
      </div>
      <h1>Tuesday Trials Grid</h1>
      {edit ? (
        <EditGrid
          db={db}
          customData={customData}
          setCustomData={setCustomData}
        />
      ) : (
        <Grid key={customData} db={db} customData={customData} />
      )}
    </>
  );
}

export default App;
