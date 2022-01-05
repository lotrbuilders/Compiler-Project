int strlen(char *);

int hash(char *str)
{
    long len = strlen(str);
    long acc = '\'';
    for (long i = 0; i < len; i = i + 1)
    {
        acc = acc + str[i] * 8 + acc / 40000000 + i;
    }

    return acc;
}

int main()
{
    return hash("\n\t\a\b\v\\%Hello worldAndSomeMore\"Text,Maybe a Z as well\'");
}