import classNames from "classnames"
import { useEffect, useState } from "react"
import { Database } from "sql.js"

function Square({select, chosen, selected, db}: {select: () => void, chosen: string | null, selected: boolean, db: Database}) {
    const [details, setDetails] = useState<{name: string | undefined, pfp: string | undefined, prefix: string | undefined} | undefined>(undefined);
    useEffect(() => {
        setDetails(db.exec("SELECT Name, ProfileUrl, Prefix FROM Player WHERE DisplayName = ? AND Main = 1", [chosen]).at(0)?.values.map(v => ({name: v.at(0)?.toString(), pfp: v.at(1)?.toString(), prefix: v.at(2)?.toString()})).at(0));
    }, [chosen])
    let displayName = chosen;
    let subName = undefined;
    if (details?.name && details.name !== chosen)
        subName = details.name;
    if ((details?.prefix?.length ?? 0) > 0)
        if (subName) subName = details?.prefix + " | " + subName;
        else displayName = details?.prefix + " | " + displayName;

    return <div className={classNames("gridSquare", {"selected": selected, "guessed": displayName != null, "smaller": (displayName?.length ?? 0) > 10})} onClick={chosen == null? select : undefined}>
        {details?.pfp && <img src={details.pfp} />}
        <h2 className="gridLabel" hidden={!chosen}>{displayName}</h2>
        {details?.name && details.name !== chosen && <p className="gridSubLabel" style={{color: "lightgray"}}>({subName})</p>}
    </div>
}
export default Square