#!/bin/bash
parent_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )

cd "$parent_path"
arm-none-eabi-gcc -nostdlib -Wl,-Tmemory.x src/main.c -march=armv4t -mthumb-interwork  -e main -o target/usercode_c.o