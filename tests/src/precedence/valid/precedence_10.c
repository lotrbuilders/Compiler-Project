int (*a[2])(int);

int b(int a)
{
    return a * 2;
}

int c(int a)
{
    return a - 2;
}

int main(void)
{
    a[0] = b;
    a[1] = c;
    return (*a)(2) + a[1](15) * a[0](1);
}