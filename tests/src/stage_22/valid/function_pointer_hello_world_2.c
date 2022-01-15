int puts(char *format);
void hello_world()
{
	puts("Hello World!\n");
	return;
}

int main()
{
	void (*test)() = &hello_world;
	test();
	return 34;
}