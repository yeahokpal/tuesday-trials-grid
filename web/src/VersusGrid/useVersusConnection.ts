import { useEffect, useState } from "react";
import VersusData from "./VersusData";
import TrialsGrid from "../TrialsGrid";
import { Database } from "sql.js";
export interface VersusConnection {
    id: string | null;
    setId: React.Dispatch<React.SetStateAction<string | null>>;
    events: EventSource | null;
    data: VersusData | null;
    sendData: (newData: VersusData | null) => void;
    host: boolean | null;
    setHost: React.Dispatch<React.SetStateAction<boolean | null>>;
    connected: boolean;
}

export function useVersusConnection(db: Database | null): VersusConnection {
    const [host, setHost] = useState<boolean | null>(null);
    const [id, setId] = useState<string | null>(null);
    const [events, setEvents] = useState<EventSource | null>(null);
    const [data, setData] = useState<VersusData | null>(null);

    let initEvents = () => {
        if (!id) return;
        let e = new EventSource("https://trials-grid-716349156143.us-central1.run.app/game/events?" +
            new URLSearchParams({id}), {
            withCredentials: true
        });
        e.onmessage = m => {
            setData(JSON.parse(m.data));
        };
        e.onerror = _ => {
            initEvents();
        };
        setEvents(e);
    };
    let sendData = (newData: VersusData | null) => {
        if (events && data && id) {
            fetch("https://trials-grid-716349156143.us-central1.run.app/game/update?" +
                new URLSearchParams({id}), {
                method: "POST",
                mode: "cors",
                body: JSON.stringify(data),
            });
        }
        setData(newData);
    };

    useEffect(() => {
        if (id && !events && host !== null && db) {
            if (host) {
                if (!data) {
                    setData(new VersusData(TrialsGrid.getRandomValidGrid(db, new Date().toISOString()).data));
                } else {
                    fetch("https://trials-grid-716349156143.us-central1.run.app/game/create?" +
                        new URLSearchParams({id}), {
                        method: "POST",
                        mode: "cors",
                        body: JSON.stringify(data),
                    }).then(_ => {
                        initEvents();
                    });
                }
            }
            else {
                initEvents();
            }
        }
    }, [id, host, data, events]);

    return {id, setId, events, data, sendData, host, setHost, connected: (events && data && id && host) != null};
}