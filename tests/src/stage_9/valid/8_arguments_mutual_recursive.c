int test();
int test1();

int main()
{
    return test(1, 2, 3, 4, 5, 6, 7, 8);
}

int test(int a, int b, int c, int d, int e, int f, int g, int h)
{
    return h == 0 ? a + b - c + d - e + f - g + h : test1(a, b, c, d, e, f, g, h);
}

int test1(int a, int b, int c, int d, int e, int f, int g, int h)
{
    return b * test(a, b + 1, c, d, e, f, g, h - 1);
}