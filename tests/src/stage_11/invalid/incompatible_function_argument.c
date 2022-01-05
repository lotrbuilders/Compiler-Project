int test(int *);

int test(int *a)
{
    return *a + 5;
}

int main()
{
    return test(6);
}