import { useEffect, useState } from "react";
import { Database } from "sql.js";
import TrialsGrid from "../TrialsGrid";
import "./Grid.css";
import Label from "./Label";
import Square from "./Square";
import SearchBar from "../Search/SearchBar";
import useLocalStorage from "../useLocalStorage";
import GameData from "./GameData";
import classNames from "classnames";

import { MAX_LIVES } from "../constants";

function getCustomGrid(db: Database, customData: string | null) {
  let newData: QueryData[] = JSON.parse(customData ?? "null");
  if (newData) {
    let grid = new TrialsGrid(db, newData, true);
    if (grid.valid) return grid;
  }
}
function Grid({db, customData}: {db: Database, customData: string | null}) {
  const [today] = useState(() => new Intl.DateTimeFormat("en-US").format(new Date()));
  // const [today] = useState(() => new Date().toISOString());
  const [players] = useState(() => db.exec("SELECT DISTINCT Name FROM Player ORDER BY Name")[0].values.flat().map(r => r!.toString()));
  const [selected, setSelected] = useState<number|null>(null);
  const [gameData, setGameData] = useLocalStorage<{[date: string]: GameData | undefined}>("gameData", {});
  // const [name, setName] = useLocalStorage<string|null>("name", null);
  const [statistics, setStatistics] = useState<ApiResponse | undefined>(undefined);
  const [customGameData, setCustomGameData] = useState<GameData | undefined>(undefined);

  const todayGameData: GameData = gameData[today] ?? new GameData(today);
  const {guesses, lives} = customGameData ?? todayGameData;

  useEffect(() => {setCustomGameData(customData? new GameData() : undefined)}, [customData]);

  const complete = guesses.every(c => c != null) || lives <= 0;
  const [alreadyComplete] = useState(complete);
  useEffect(() => {
    if (!customData && !statistics) {
      if (alreadyComplete) {
        fetch("https://trials-grid-716349156143.us-central1.run.app?" + new URLSearchParams({
          data: JSON.stringify({today, guesses})
        }), {
          method: "GET",
          mode: "cors"
        }).then(res => res.json()).then(json => setStatistics(json));
      } else if (complete) {
        fetch("https://trials-grid-716349156143.us-central1.run.app", {
          method: "POST",
          body: JSON.stringify({today, guesses}),
          headers: {"Content-Type": "application/json"},
          mode: "cors"
        }).then(res => res.json())
          .then(json => setStatistics(json));
      }
    }
  }, [complete]);

  const [grid] = useState(() => getCustomGrid(db, customData) ?? TrialsGrid.getRandomValidGrid(db, today));

  let choose = (guess: string): boolean | undefined => {
    if (selected === null) return;
    if (guesses[selected] != null) return;
    if (guesses.includes(guess)) return;

    if (grid.answers[selected]!.includes(guess)) {
      let newGuesses = [...guesses];
      newGuesses[selected] = guess;
      if (!customGameData) setGameData({...gameData, [today]: {...todayGameData, guesses: newGuesses}});
      else setCustomGameData({...customGameData, guesses: newGuesses});
      setSelected(null);
      return true;
    } else {
      if (!customGameData) {
        setGameData({...gameData, [today]: {...todayGameData, lives: lives - 1}});
      }
      else setCustomGameData({...customGameData, lives: lives - 1});
      return false;
    }
  }
  let hearts = [...Array(MAX_LIVES)].map((_, i) => i < lives? "❤️" : "🖤");

  function copyResults() {
    navigator.clipboard.writeText(`\
Tuesday Trials Grid ${customData? "Custom Grid" : today} ${customData? "" : hearts.join("")}
${[0, 1, 2].map(i => guesses.slice(i * 3, i * 3 + 3))
  .map(arr => arr.map(s => s? "🟩" : "⬛").join(""))
  .join("\n")
}`)
  }

  return <>
    <div className={classNames("grid", {"complete": complete})}>
      {grid.columns.map((c, i) => <div key={i} style={{gridColumn: i + 2}}><Label text={c!.label}/></div>)}
      {grid.rows.map((r, i) => <div key={i} style={{gridRow: i + 2, height: 0}}><div style={{alignContent: "center", transform: "translateY(-50%)"}}><Label text={r!.label}/></div></div>)}
      {grid.answers.map((_, i) =>
      <div key={i} style={{gridRow: i / 3 + 2, gridColumn: i % 3 + 2, position: "relative"}}>
        <div className="stat" style={{position: "absolute", right: "0"}}>{statistics?.guesses[i]? (statistics.guessesCorrect[i]! * 100 / statistics.guesses[i]).toPrecision(3) + "%" : undefined}</div>
        <Square chosen={guesses[i]} select={() => !complete && setSelected(i)} selected={selected === i}/>
      </div>)}
      <div style={{gridRow: 2, gridColumn: 5, alignContent: "center"}}>
      <h1>{hearts}</h1>
      </div>
      {complete && <div style={{gridRow: 4, gridColumn: 5, alignContent: "center"}}><button onClick={copyResults}>Copy Results</button></div>}
    </div>
    <SearchBar key={selected} choices={guesses.filter(c => c != null)} choose={choose} options={players} visible={selected !== null} hide={() => setSelected(null)}/>
  </>;
}
export default Grid