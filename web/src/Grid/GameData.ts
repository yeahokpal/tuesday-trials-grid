import { COLUMNS, MAX_LIVES, ROWS } from "../constants"

class GameData {
    date?: string
    guesses: (string | null)[] = new Array(ROWS * COLUMNS).fill(null)
    lives: number = MAX_LIVES
    constructor(date?: string) {
        this.date = date;
    }
}
export default GameData