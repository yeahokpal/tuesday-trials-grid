import classNames from "classnames"
import { useEffect, useState } from "react"
import { Database } from "sql.js"

function Square({select, chosen, selected, db}: {select: () => void, chosen: string | null, selected: boolean, db: Database}) {
    const [img, setImg] = useState<string | undefined>(undefined);
    useEffect(() => {
        setImg(db.exec("SELECT ProfileUrl FROM Player WHERE Name = ? AND Main = 1", [chosen]).at(0)?.values?.at(0)?.at(0)?.toString());
    }, [chosen])
    return <div className={classNames("gridSquare", {"selected": selected, "guessed": chosen != null, "smaller": (chosen?.length ?? 0) > 10})} onClick={select}>
        {img && <img src={img} />}
        <h2 className="gridLabel" hidden={!chosen}>{chosen}</h2>
    </div>
}
export default Square