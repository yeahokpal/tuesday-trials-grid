import queryFile from "../GridQueries.toml";
import { getQueryVars } from "../queryfuncs";
function EditLabel({data, setData}: {data?: QueryData, setData: (d: QueryData) => void}) {
    function onChangeQuery(newValue: string) {
        setData({id: newValue, v: {}})
    }
    function onChangeVariable(key: string, newValue: string) {
        setData({id: data!.id, v: {...data!.v, [key]: newValue}});
    }
    return <div>
        <select onChange={e => onChangeQuery(e.target.value)}>
            <option value=""></option>
            {Object.entries(queryFile.queries).map(([k, q]) => 
            <option key={k} value={k}>{q.label}</option>
            )}
        </select>
        {data && getQueryVars(queryFile, queryFile.queries[data.id]).map(([k, v]) =>
            <select key={k} onChange={e => onChangeVariable(k, e.target.value)}>
                <option value=""></option>
                {(v.values?.map(val => val.id) ?? v.strValues ?? []).map(val =>
                <option key={val} value={val}>{val}</option>
                )}
            </select>
        )}
    </div>;
}
export default EditLabel