#include <time.h>
#include <stdlib.h>
#include <stdio.h>

int gcd(int, int);

int main()
{
    int a = 987654321;
    int b = 123456789;
    clock_t start = clock();
    for (int i = 0; i < 100; i = i + 1)
    {
        gcd(a, b);
    }
    clock_t end = clock();
    double diff = (double)end - (double)start;
    double t_m = (diff) / CLOCKS_PER_SEC * 1000;
    printf("time used: %.2f ms\n", t_m);
}