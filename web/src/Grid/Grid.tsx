import { useEffect, useState } from "react";
import { Database } from "sql.js";
import TrialsGrid from "../TrialsGrid";
import "./Grid.css";
import Label from "./Label";
import Square from "./Square";
import SearchBar from "../Search/SearchBar";
import useLocalStorage from "../useLocalStorage";
import GameData from "../GameData";
import classNames from "classnames";

import { MAX_LIVES } from "../constants";
import Results from "../Results/Results";
import { calcLocalStats } from "../calcLocalStats";
import { useMediaQuery } from "@mui/material";
import pastQueryData from "../pastQueryData";

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
  const [localData, setLocalData] = useLocalStorage<LocalData>("gameData", {});
  const [name, setName] = useLocalStorage<string|null>("name", null);
  const [statistics, setStatistics] = useState<ApiResponse | undefined>(undefined);
  const [customGameData, setCustomGameData] = useState<GameData | undefined>(undefined);

  const todayGameData: GameData = localData[today] ?? new GameData(today);
  const {guesses, lives} = customGameData ?? todayGameData;


  const complete = guesses.every(c => c != null) || lives <= 0;
  const [alreadyComplete] = useState(complete);
  const [resultsVisible, setResultsVisible] = useState(false);

  useEffect(() => {setCustomGameData(customData? new GameData() : undefined)}, [customData]);
  useEffect(() => setResultsVisible(statistics !== undefined), [statistics]);
  useEffect(() => {selected && complete && setSelected(null)}, [selected && complete]);
  useEffect(() => {
    if (!customData && !statistics) {
      if (alreadyComplete) {
        fetch("https://trials-grid-716349156143.us-central1.run.app?" +
          new URLSearchParams({data: JSON.stringify({today, guesses})}), {
          method: "GET",
          mode: "cors"
        }).then(res => res.json()).then(json => setStatistics(json));
      } else if (complete) {
        fetch("https://trials-grid-716349156143.us-central1.run.app", {
          method: "POST",
          body: JSON.stringify({today, name, guesses}),
          headers: {"Content-Type": "application/json"},
          mode: "cors"
        }).then(res => res.json())
          .then(json => setStatistics(json));
      }
    }
  }, [complete]);
  useEffect(() => {
    if (!customData && name && statistics?.id) {
      fetch("https://trials-grid-716349156143.us-central1.run.app?" + 
        new URLSearchParams({id: statistics.id, today}), {
        method: "POST",
        body: name,
        mode: "cors"
      });
    }
  }, [name]);

  const [grid] = useState(() => getCustomGrid(db, customData) ?? (pastQueryData[today]? new TrialsGrid(db, pastQueryData[today], false) : TrialsGrid.getRandomValidGrid(db, today)));

  let choose = (guess: string): boolean | undefined => {
    if (selected === null) return;
    if (guesses[selected] != null) return;
    if (guesses.includes(guess)) return;

    if (grid.answers[selected]!.includes(guess)) {
      let newGuesses = [...guesses];
      newGuesses[selected] = guess;
      if (!customGameData) setLocalData({...localData, [today]: {...todayGameData, guesses: newGuesses}});
      else setCustomGameData({...customGameData, guesses: newGuesses});
      setSelected(null);
      return true;
    } else {
      if (!customGameData) {
        setLocalData({...localData, [today]: {...todayGameData, lives: lives - 1}});
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

  const mobile = useMediaQuery('(max-width: 1200px)');


  return <>
    <div className={classNames("grid", {"complete": complete})}>
      {grid.columns.map((c, i) => <div key={i} style={{gridColumn: i + 2}}><Label text={c!.label}/></div>)}
      {grid.rows.map((r, i) => <div key={i} style={{gridRow: i + 2, height: 0}}><div style={{alignContent: "center", transform: "translateY(-50%)"}}><Label text={r!.label}/></div></div>)}
      {grid.answers.map((_, i) =>
      <div key={i} style={{gridRow: i / 3 + 2, gridColumn: i % 3 + 2, position: "relative"}}>
        {statistics?.guesses[i] != null && 
        <div className="stat">
          {(statistics.guesses[i] === 1 ? 
            "Unique!" :
            (statistics.guesses[i] * 100 / statistics.totalGuesses[i]).toPrecision(3) + "%")}
        </div>
        }
        <Square chosen={guesses[i]} select={() => !complete && setSelected(i)} selected={selected === i} db={db}/>
      </div>)}
      <div style={{gridRow: (mobile? 5 : 2), gridColumn: (mobile? 1 : 5), alignContent: "center", gridColumnEnd: (mobile? 5 : undefined)}}>
        <h1>{hearts}</h1>
      </div>
      {complete && <div style={{gridRow: (mobile? 6 : 4), gridColumn: (mobile? 1 : 5), alignContent: "center", gridColumnEnd: (mobile? 5 : undefined)}}>
        <button onClick={copyResults}>Copy Results</button>
        <button onClick={() => setResultsVisible(true)}>View Stats</button>
      </div>}
    </div>
    <SearchBar key={selected} choices={guesses.filter(c => c != null)} choose={choose} options={players} visible={selected !== null} hide={() => setSelected(null)}/>
    {resultsVisible && <Results
        visible={resultsVisible}
        hide={() => setResultsVisible(false)}
        localStats={calcLocalStats(today, localData)}
        statistics={statistics!}
        name={name}
        setName={setName}
        copyResults={copyResults}/>}
  </>;
}
export default Grid