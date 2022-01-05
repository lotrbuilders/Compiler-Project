int *malloc(int size);

int main()
{
    int *ptr = malloc(8 * 4);
    return (ptr + 3) - ptr;
}