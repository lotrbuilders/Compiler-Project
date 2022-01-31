#include <time.h>
#include <stdlib.h>
#include <stdio.h>

char *primes(char *, size_t len);

int main()
{
    int len = 100000;
    char *prime = malloc(len);
    clock_t start = clock();
    char *p = primes(prime, len);
    clock_t end = clock();
    double diff = (double)end - (double)start;
    double t_m = (diff) / CLOCKS_PER_SEC * 1000;
    printf("time used: %.2f ms\n", t_m);
}