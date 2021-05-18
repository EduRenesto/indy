TARGETS := 90.simple.o 91.api.elf
CFLAGS := -g -march=mips1 -mcompact-branches=never -O0 \
	-fno-lto -nostdlib -mno-shared -ffreestanding \
	-static -fno-pie -fno-pic

%.o: %.c
	mipsel-linux-gnu-gcc ${CFLAGS} $^ -nostdlib -c -o $@

%.o: %.S
	mipsel-linux-gnu-gcc ${CFLAGS} $^ -fPIC -nostdlib -c -o $@

%.elf: indy_api.o %.o
	#mipsel-linux-gnu-gcc ${CFLAGS} -T linker.ld -nostdlib $^ -o $@
	mipsel-linux-gnu-gcc ${CFLAGS} -nostdlib $^ -o $@

all: ${TARGETS}

clean:
	rm *.o
	rm *.elf

.PRECIOUS: %.o

.PHONY: clean
