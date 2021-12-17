int *malloc(int size);

int main()
{
	int *ptr = malloc(16);
	*ptr = 2;
	return *ptr;
}