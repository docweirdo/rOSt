#include <stddef.h>

const unsigned int DBGU_SERVICE = 10;

void subscribe_to_thread_service(unsigned int service)
{
    asm("swi #34");
}

char receive_char_from_dbgu()
{
    char out;
    asm("mov r0, #1;" // blocking
        "swi #11;"
        "mov %0, r0;" : "=r"(out) :: "%r0");
    return out;
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
    subscribe_to_thread_service(DBGU_SERVICE);
    while (1) {
        char received = receive_char_from_dbgu();

        if (received == 'q') {
            break;
        }
        if (received == 't') {
            print_string("tip\n");
            yield_thread();
            print_string("top\n");
        } else {
            print_string("no: ");
            send_char_to_dbgu(received);
            send_char_to_dbgu('\n');
        }
    }
    return 0;
}