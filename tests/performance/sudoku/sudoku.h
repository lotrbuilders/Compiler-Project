#define SUDOKU_SIZE 9

void getSudoku(char *filename);
void printSudoku(int sudoku[9][9]);

int isPossible(int x, int y, int n);
void solve(int x, int y);

int finished();