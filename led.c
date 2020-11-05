#define PIOB 0xfffff600
#define YELLOW_LED (1 << 27)
struct pio
{
    unsigned int per;
    unsigned int unused0[3];
    unsigned int oer;
    unsigned int unused1[7];
    unsigned int sodr;
};
static volatile struct pio *const piob = (struct pio *)PIOB;
void yellow_on(void)
{
    /* Initialisieren */
    piob->per = YELLOW_LED;
    piob->oer = YELLOW_LED;
    /* Anschalten */
    piob->sodr = YELLOW_LED;
}

//__attribute__((naked, section(".init")))
void _start(void) {
yellow_on();
for(;;);
}
