import queryFile from "../GridQueries.toml";
import { getQueryVars } from "../queryfuncs";
function EditLabel({data, setData}: {data?: QueryData, setData: (d: QueryData) => void}) {
    function onChangeQuery(newValue: string) {
        setData({i: Number.parseInt(newValue), v: {}})
    }
    function onChangeOption(newValue: string) {
        setData({i: data!.i, v: JSON.parse(newValue)})
    }
    function onChangeVariable(key: string, newValue: string) {
        setData({i: data!.i, v: {...data!.v, [key]: newValue}});
    }
    return <div>
        <select onChange={e => onChangeQuery(e.target.value)}>
            <option value=""></option>
            {queryFile.queries.map((q, i) => 
            <option key={i} value={i}>{q.label}</option>
            )}
        </select>
        {data && (queryFile.queries[data.i].options?
            <select onChange={e => onChangeOption(e.target.value)}>
                <option value="{}"></option>
                {queryFile.queries[data.i].options?.map(opt => JSON.stringify(opt)).map(str => 
                <option value={str}>{str}</option>
                )}
            </select>
        : getQueryVars(queryFile, data.i).map(([k, v]) =>
            <select key={data.i + k} onChange={e => onChangeVariable(k, e.target.value)}>
                <option value=""></option>
                {[...new Set(v.values)].map((val, i) =>
                <option value={val}>{queryFile.vars[k].labels?.at(i) ?? val}</option>
                )}
            </select>
        ))}
    </div>;
}
export default EditLabel