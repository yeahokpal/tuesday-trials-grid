import { useState } from "react";

function SearchOption({value, choose}: {value: string, choose: ((s: string) => boolean | undefined) | undefined}) {
    const [result, setResult] = useState<boolean | undefined>(undefined);
    const option = (result === false? "Nope!" : (choose ? "Choose" : "Used"));
    
    return <div className="searchOption" style={{display: "flex"}}>
        <h3 style={{flexGrow: 1}}>{value}</h3>
        <button style={{alignSelf: "center", marginLeft: "20px"}} disabled={choose === undefined || result === false} onClick={() => setResult(choose?.(value))}>
            {option}
        </button>
    </div>
}
export default SearchOption