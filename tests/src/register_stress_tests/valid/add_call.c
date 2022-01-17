int test(int a, int b)
{
    return a + b;
}

int main()
{
    return 1 + (2 + (test(3, 16) + (4 + (5 + (6 + (7 + (8 + (9 + (10 + (11 + (12 + (13 + (test(14, 15))))))))))))));
}
