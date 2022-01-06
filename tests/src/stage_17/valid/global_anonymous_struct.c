char *calloc(int n, int size);
struct
{
    char c;
    int i;
} b;

int main()
{
    b.c = 5 + 2;
    b.i = 20;
    return b.c * b.i;
}