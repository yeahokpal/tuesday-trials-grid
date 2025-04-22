interface GameData {
    date?: string
    guesses: (string | null)[];
    lives: number;
}

interface QueryFile {
    queries: Query[];
    vars: {[key: string]: Var};
}

interface Query {
    label: string;
    query: string;
    options?: {[k: string]: string}[]
    vars?: {[k: string]: string}
    odds: number;
}
interface Var {
    values?: string[];
    labels?: string[];
    query?: string;
}
interface QueryData {
    i: number;
    v: {[k: string]: string};
}

declare module "*.toml" {
    const value: QueryFile;
    export default value;
}

interface ApiResponse {
    id?: string,
    total: number,
    totalCorrect: number,
    guesses: (number|null)[],
    totalGuesses: number[],
}

interface LocalData {
    [date: string]: GameData | undefined
}

interface LocalStats {
    todayGameData: GameData,
    gameCount: number,
    gamesWon: number,
    streak: number
}