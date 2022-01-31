#include <time.h>
#include <stdlib.h>
#include <stdio.h>
#include "sudoku.h"

int mainSudoku[9][9] = {0};
int horizontals[9][9] = {0};
int verticals[9][9] = {0};
int blocks[3][3][9] = {0};

clock_t start;

int main()
{
    getSudoku("./tests/performance/sudoku/sudoku.sdk");
    start = clock();
    solve(0, 0);
    return -1;
}

int finished()
{
    clock_t end = clock();
    double diff = (double)end - (double)start;
    double t_m = (diff) / CLOCKS_PER_SEC * 1000;
    printf("time used: %.2f ms\n", t_m);
    exit(0);
}