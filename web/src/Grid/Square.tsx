import classNames from "classnames"
import { useEffect, useState } from "react"
import { Database } from "sql.js"

function Square({select, chosen, selected, db}: {select: () => void, chosen: string | null, selected: boolean, db: Database}) {
    const [details, setDetails] = useState<{name: string | undefined, pfp: string | undefined} | undefined>(undefined);
    useEffect(() => {
        setDetails(db.exec("SELECT Name, ProfileUrl FROM Player WHERE DisplayName = ? AND Main = 1", [chosen]).at(0)?.values.map(v => ({name: v.at(0)?.toString(), pfp: v.at(1)?.toString()})).at(0));
    }, [chosen])
    return <div className={classNames("gridSquare", {"selected": selected, "guessed": chosen != null, "smaller": (chosen?.length ?? 0) > 10})} onClick={chosen == null? select : undefined}>
        {details?.pfp && <img src={details.pfp} />}
        <h2 className="gridLabel" hidden={!chosen}>{chosen}</h2>
        {details?.name && details.name !== chosen && <p className="gridSubLabel" style={{color: "lightgray"}}>({details.name})</p>}
    </div>
}
export default Square