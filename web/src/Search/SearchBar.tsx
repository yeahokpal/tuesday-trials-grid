import { useState } from "react";
import "./SearchBar.css"
import SearchOption from "./SearchOption";
function SearchBar({choices, choose, options, visible, hide}: 
        {
            choices: string[],
            choose: (choice: string) => boolean | undefined, 
            options: string[],
            visible: boolean,
            hide: () => void,
        }) {
    const [input, setInput] = useState("");
    return <div className="searchModal" onClick={() => hide()} hidden={!visible}>
        <div className="search" onClick={e => e.stopPropagation()}>
            <input ref={input => input?.focus()} 
                className="searchBar" 
                value={input} 
                onChange={e => setInput(e.target.value)} 
                onKeyDown={e => e.key === "Escape" && hide()}
                autoFocus/>
            {/^\s*$/.test(input)? <></> :
                options.filter(o => (o.toLowerCase().includes(input.toLowerCase())))
                    .map(o => <SearchOption key={o} value={o} choose={choices.includes(o)? undefined : choose}/>)
            }
        </div>
    </div>
}
export default SearchBar