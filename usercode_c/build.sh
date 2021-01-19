#!/bin/sh

arm-none-eabi-gcc -nostdlib -Wl,-Tmemory.x src/test.c -march=armv4t -mthumb-interwork  -e main -o target/usercode_c.o