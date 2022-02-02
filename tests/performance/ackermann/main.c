#include <time.h>
#include <stdlib.h>
#include <stdio.h>

long ackermann(long, long);

int main()
{
    clock_t start = clock();
    ackermann(4, 1);
    clock_t end = clock();
    double diff = (double)end - (double)start;
    double t_m = (diff) / CLOCKS_PER_SEC * 1000;
    printf("time used: %.2f ms\n", t_m);
}