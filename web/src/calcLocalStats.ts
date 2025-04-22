export function calcLocalStats(today: string, gameData: LocalData): LocalStats {
    const format = new Intl.DateTimeFormat("en-US");
    let streak = 0;
    for(let date = new Date(today);
        gameData[format.format(date)]?.guesses.every(g => g !== null);
        streak++, date.setDate(date.getDate() - 1));

    return {
        todayGameData: gameData[today]!,
        gameCount: Object.keys(gameData).length,
        gamesWon: Object.entries(gameData).filter(([_, data]) => data?.guesses.every(g => g !== null)).length,
        streak
    }
}