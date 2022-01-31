#include "sudoku.h"
void *fopen(char *, char *);
void fgets(char *, int, void *);
int printf();
void exit(int);
int puts(char *);

int mainSudoku[9][9];
int horizontals[9][9];
int verticals[9][9];
int blocks[3][3][9];

void getSudoku(char *filename)
{

    void *file = fopen(filename, "r");
    if (!file)
    {
        printf("File doesn't exist\n");
        exit(-1);
    }

    char string[128];
    fgets(string, 82, file);

    for (int j = 0; j < 9; j = j + 1)
    {

        for (int i = 0; i < 9; i = i + 1)
        {

            if (string[j * 9 + i] == '.')
                mainSudoku[j][i] = 0;
            else
                mainSudoku[j][i] = string[j * 9 + i] - '0';
        }
    }
    for (int i = 0; i < 9; i = i + 1)
    {

        for (int j = 0; j < 9; j = j + 1)
        {

            int n = mainSudoku[i][j];
            if (n != 0)
                horizontals[i][n - 1] = n;

            n = mainSudoku[j][i];
            if (n != 0)
                verticals[i][n - 1] = n;

            n = mainSudoku[i][j];
            if (n != 0)
            {

                blocks[j / 3][i / 3][n - 1] = n;
            }
        }
    }
}

int isPossible(int x, int y, int n)
{
    int k = n - 1;
    if (mainSudoku[y][x] != 0)
    {
        return 0;
    }

    if (horizontals[y][k] || verticals[x][k] || blocks[x / 3][y / 3][k])
    {
        return 0;
    }
    else
    {
        return 1;
    }
}

void solve(int x, int y)
{
    for (; x < 9; x = x + 1)
    {
        for (; y < 9; y = y + 1)
        {
            if (mainSudoku[y][x] == 0)
            {
                for (int n = 1; n < 10; n = n + 1)
                {
                    if (isPossible(x, y, n))
                    {
                        mainSudoku[y][x] = n;
                        horizontals[y][n - 1] = n;
                        verticals[x][n - 1] = n;
                        blocks[x / 3][y / 3][n - 1] = n;

                        solve(x, y);

                        mainSudoku[y][x] = 0;
                        horizontals[y][n - 1] = 0;
                        verticals[x][n - 1] = 0;
                        blocks[x / 3][y / 3][n - 1] = 0;
                    }
                }
                return;
            }
        }
        y = 0;
    }
    finished();

    return;
}
