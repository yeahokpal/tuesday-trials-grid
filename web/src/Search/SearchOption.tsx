import { useState } from "react";

function SearchOption({value, choose}: {value: {name: string, displayName: string}, choose: ((s: string) => boolean | undefined) | undefined}) {
    const [result, setResult] = useState<boolean | undefined>(undefined);
    const option = (result === false? "Nope!" : (choose ? "Choose" : "Used"));
    
    return <div className="searchOption" style={{display: "flex"}}>
        <h3 style={{flexGrow: 1}}>{value.displayName}</h3>
        {value.name !== value.displayName? <h4 style={{flexGrow: 1, alignSelf: "center", color: "lightgray"}}>({value.name})</h4> : <></>}
        <button style={{alignSelf: "center", marginLeft: "20px"}} disabled={choose === undefined || result === false} onClick={() => setResult(choose?.(value.displayName))}>
            {option}
        </button>
    </div>
}
export default SearchOption