int test(int a, int, int b);

int test(int d, int b, int a)
{
    return d * (5 + a) + b;
}

int main()
{
    return test(6, 3, 2);
}