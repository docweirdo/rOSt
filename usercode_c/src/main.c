#include <stddef.h>

void receive_char_from_dbgu(char c)
{
    asm("swi #11");
}

void send_char_to_dbgu(char c)
{
    asm("swi #10");
}

void yield_thread()
{
    asm("swi #32");
}

void print_string(const char *chars)
{
    while (chars != NULL && *chars != '\0')
    {
        send_char_to_dbgu(*chars);
        chars++;
    }
}

int __attribute__((section(".text.main"))) main()
{
    print_string("tip");
    yield_thread();
    print_string("top");
    return 0;
}