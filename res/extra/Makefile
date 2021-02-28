TARGETS := 90.simple

%.text: %.o
	mipsel-linux-gnu-objcopy -O binary --only-section=.text $^ $@

%.data: %.o
	mipsel-linux-gnu-objcopy -O binary --only-section=.data $^ $@

%.o: %.c
	mipsel-linux-gnu-gcc $^ -nostdlib -o $@ -T linker.ld

%.o: %.S
	mipsel-linux-gnu-gcc $^ -nostdlib -o $@ -T linker.ld

all: ${TARGETS}.data ${TARGETS}.text

.PRECIOUS: %.o
