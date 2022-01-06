char *calloc(int n, int size);

int main()
{
    struct
    {
        char c;
        int i;
    } b;
    b.c = 5 + 2;
    b.i = 20;
    return b.c * b.i;
}