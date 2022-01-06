char *calloc(int n, int size);

int main()
{
    {
        struct test
        {
            char c;
            int i;
        };
    }

    struct test b;
    b.c = 5 + 2;
    b.i = 20;
    return b.c * b.i;
}