#define mod(a, b) ((a) - (b) * ((a) / (b)))
int is_prime(int a)
{
    for (int i = 2; i < a; i = i + 1)
    {
        if (mod(a, i) == 0)
        {
            return 0;
        }
    }
    return 1;
}

char *primes(char *primes, long len)
{

    for (long i = 3; i < len; i = i + 2)
    {
        primes[i] = is_prime(i);
    }
    return primes;
}