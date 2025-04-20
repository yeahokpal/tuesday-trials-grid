import classNames from "classnames"

function EditSquare({answers}: {answers?: string[]}) {
    return <div className={classNames("gridSquare", "editSquare")} style={{alignContent: "center", wordWrap: "break-word"}}>
        <h2 className="gridLabel">{answers?.length ?? ""}</h2>
    </div>
}
export default EditSquare