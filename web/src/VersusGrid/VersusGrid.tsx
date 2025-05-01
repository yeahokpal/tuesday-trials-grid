import { useState } from "react";
import { Database } from "sql.js";
import TrialsGrid from "../TrialsGrid";
import "../Grid/Grid.css";
import Label from "../Grid/Label";
import Square from "../Grid/Square";
import SearchBar from "../Search/SearchBar";
import classNames from "classnames";

// import { useMediaQuery } from "@mui/material";
import { VersusConnection } from "./useVersusConnection";

function VersusGrid({db, connection}: {
      db: Database, 
      connection: VersusConnection}) {
  const [players] = useState(() => db.exec("SELECT DISTINCT Name FROM Player ORDER BY Name")[0].values.flat().map(r => r!.toString()));
  const [selected, setSelected] = useState<number|null>(null);
  // const [name, setName] = useLocalStorage<string|null>("name", null);

  const grid = new TrialsGrid(db, connection.data!.data, true);

  const host = connection.host!;
  const {guesses, turn, used} = connection.data!;


  let choose = (guess: string): boolean | undefined => {
    if (selected === null) return;
    if (guesses[selected]?.turn === host) return;
    if (used.includes(guess)) return;

    if (grid.answers[selected]!.includes(guess)) {
      let newGuesses = [...guesses];
      let newUsed = used;
      newGuesses[selected] = {guess, turn: host};
      newUsed.push(guess);
      connection.sendData({...connection.data!, guesses: newGuesses, used: newUsed, turn: !host});
      setSelected(null);
      return true;
    } else {
      connection.sendData({...connection.data!, turn: !host});
      return false;
    }
  }

  // const mobile = useMediaQuery('(max-width: 1200px)');

  return <>
    <div className={classNames("grid")}>
      {grid.columns.map((c, i) => <div key={i} style={{gridColumn: i + 2}}><Label text={c!.label}/></div>)}
      {grid.rows.map((r, i) => <div key={i} style={{gridRow: i + 2, height: 0}}><div style={{alignContent: "center", transform: "translateY(-50%)"}}><Label text={r!.label}/></div></div>)}
      {grid.answers.map((_, i) =>
      <div key={i} style={{gridRow: i / 3 + 2, gridColumn: i % 3 + 2, position: "relative"}} className={classNames({host: guesses[i]?.turn === true, client: guesses[i]?.turn === false})}>
        <Square chosen={guesses[i]?.guess || null} select={() => (host === turn) && (guesses[i]?.turn !== host) && setSelected(i)} selected={selected === i} db={db}/>
      </div>)}
    </div>
    <SearchBar key={selected} used={used} choose={choose} options={players} visible={selected !== null} hide={() => setSelected(null)}/>
  </>;
}
export default VersusGrid