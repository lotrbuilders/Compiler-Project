int puts(char *format);
int hello_world()
{
	return puts("Hello World!\n");
}

int main()
{
	int (*test)() = &hello_world;
	return test();
}