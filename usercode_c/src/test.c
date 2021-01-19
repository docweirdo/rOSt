void print_char(char c) {
    asm("swi #10");
}

void  __attribute__ ((section (".text.main"))) main() {
    print_char('1');
    print_char('2');
    print_char('3');
    print_char('4');
    print_char('5');
    print_char('6');
}