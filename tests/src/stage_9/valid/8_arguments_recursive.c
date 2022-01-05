int test();

int main()
{
    return test(1, 2, 3, 4, 5, 6, 7, 8);
}

int test(int a, int b, int c, int d, int e, int f, int g, int h)
{
    if (h == 0)
    {
        return a + b - c + d - e + f - g + h;
    }
    return b * test(a, b + 1, c, d, e, f, g, h - 1);
}