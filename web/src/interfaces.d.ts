interface GameData {
    date?: string
    guesses: (string | null)[]
    lives: number
}

interface QueryFile {
    queries: {[key: string]: Query}
    vars: {[key: string]: Var}
}

interface Query {
    label: string
    query: string
    odds: number
}
interface VarValue {
    id: string
    [key: string]: string
    odds?: number
}
interface Var {
    strValues?: string[]
    values?: VarValue[]
    labels?: string[]
    query?: string
}
interface QueryData {
    id: string
    v: {[k: string]: string}
}

declare module "*/GridQueries.toml" {
    const value: QueryFile;
    export default value;
}

interface ApiResponse {
    id?: string,
    total: number,
    totalCorrect: number,
    guesses: (number|null)[],
    totalGuesses: number[],
    weekly: {name: string, count: number}[],
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
interface EventData
{
    id: string,
    name: string,
    startTime: number,
    endTime: number,
    duration: number
}