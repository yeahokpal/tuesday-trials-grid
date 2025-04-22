import classNames from "classnames"

function Square({select, chosen, selected}: {select: () => void, chosen: string | null, selected: boolean}) {
    return <div className={classNames("gridSquare", {"selected": selected, "guessed": chosen != null})}  onClick={chosen == null? select : undefined}>
        <h2 className="gridLabel" hidden={!chosen}>{chosen}</h2>
    </div>
}
export default Square