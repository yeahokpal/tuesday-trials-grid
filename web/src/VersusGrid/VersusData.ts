import { COLUMNS, ROWS } from "../constants";

class VersusData {
    guesses: ({guess: string, turn: boolean} | null)[] = Array.from(new Array(ROWS * COLUMNS).fill(null));
    turn: boolean = true;
    data: QueryData[];
    used: string[] = [];
    constructor(data: QueryData[]) {
        this.data = data;
    }
}
export default VersusData