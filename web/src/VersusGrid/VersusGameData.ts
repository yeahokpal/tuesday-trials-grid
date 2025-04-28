import { COLUMNS, ROWS } from "../constants";

class VersusGameData {
    answers: ({answer: string, host: boolean}[] | null)[] = Array.from(new Array(ROWS * COLUMNS).fill(null));
    hostTurn: boolean;
    constructor(hostTurn: boolean) {
        this.hostTurn = hostTurn;
    }
}
export default VersusGameData