interface ApiResponse {
    total: number,
    totalCorrect: number,
    guesses: (number|null)[],
    guessesCorrect: (number|null)[],
}