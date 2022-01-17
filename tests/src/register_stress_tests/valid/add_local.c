int test(int a, int b)
{
    return 1 + (2 + (a + (4 + (5 + (6 + (7 + (8 + (9 + (10 + (11 + (12 + (13 + (14 + b)))))))))))));
}

int main()
{
    return test(3, 15);
}
