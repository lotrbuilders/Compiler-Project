int main()
{
    int a = 4;
    int *b = &a;
    b | 2;
    return *b;
}