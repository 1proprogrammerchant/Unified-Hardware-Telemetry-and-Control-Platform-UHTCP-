CC = gcc
CFLAGS = -Iinclude -O2 -fPIC
SRC = src/hal.c src/hal_gpio.c src/hal_cpu.c src/hal_memory.c src/hal_serial.c
OBJ = $(SRC:.c=.o)

.PHONY: all clean

all: libuhtcp_hal.a

libuhtcp_hal.a: $(OBJ)
	$(AR) rcs $@ $^

clean:
	rm -f $(OBJ) libuhtcp_hal.a
