import "./Results.css"
import classNames from "classnames";

function Results({visible, hide, localStats, statistics, name, setName, copyResults}: {
        visible: boolean, 
        hide: () => void, 
        localStats: LocalStats,
        statistics: ApiResponse,
        name: string | null,
        setName: (name: string) => void,
        copyResults: () => void
}) {
    function onSubmit(e: React.FormEvent<HTMLFormElement>) {
        e.preventDefault();
        let el = e.currentTarget.elements.namedItem("name");
        let val = (el as HTMLInputElement)?.value;
        if (val) setName(val);
    }
    const guessPct = statistics.totalGuesses.map(g => (g * 100 / statistics.total).toFixed(0) + "%");
    const todayGameData = localStats.todayGameData;
    return <div className="resultsModal" onClick={() => hide()} hidden={!visible}
                onKeyDown={e => e.key === "Escape" && hide()}>
        <div className="results" onClick={e => e.stopPropagation()}>
            <h2>{todayGameData.guesses.every(c => c !== null)? "Congratulations!" : "Better Luck Tomorrow..."}</h2>
            <h3>{(statistics.totalCorrect * 100 / statistics.total).toPrecision(3)}% of people completed today's grid.</h3>
            <div className="resultsGrid">
                {guessPct.map((g, i) => <div className={classNames("resultsCell", {guessed: todayGameData.guesses[i] !== null})} style={{gridRow: Math.floor(i / 3 + 1), gridColumn: i % 3 + 1}}>{g}</div>)}
            </div>
            <div style={{display: "grid"}}>
                {name? <>
                    <h4>{name}</h4>
                    <h4 style={{gridRow: 2}}>Games Won: {localStats.gamesWon}<br/>
                    Streak: {localStats.streak}</h4>
                    </>
                    : <>
                    <label htmlFor="name">Name: </label>
                    <form style={{gridRow: 2}} onSubmit={onSubmit}><input id="name"/></form>
                    </>
                }
                <button style={{gridColumn: 2, gridRowStart: 1, gridRowEnd: 3}} onClick={copyResults}>Copy Results</button>
            </div>
        </div>
    </div>
}
export default Results