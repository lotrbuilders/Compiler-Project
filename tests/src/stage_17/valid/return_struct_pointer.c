char *calloc(int n, int size);
struct test
{
	int a;
	short c;
};

struct test *tmp(int a, long b)
{
	struct test *ptr = ((struct test *)calloc(1, sizeof(struct test)));
	ptr->a = a - b;
	ptr->c = b - a;
	return ptr;
}

int main()
{
	struct test *ptr = tmp(4, 6);
	return ptr->a * ptr->c;
}