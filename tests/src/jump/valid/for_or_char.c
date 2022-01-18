int main()
{
    int a = 1;
    char cont = 0;
    for (int i = 1; i < 10 || cont; i = i + 1)
    {
        cont = i == 9 ? 1 : 0;
        a = a * i;
    }
    return a;
}