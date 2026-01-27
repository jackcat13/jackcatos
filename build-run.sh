nasm -f bin boot/boot.asm -o out/boot.bin
nasm -f elf64 boot/kernel_entry.asm -o out/kernel_entry.o

cargo rustc --release -- --emit obj=out/kernel.o

ld.lld -T linker.ld -o out/kernel.bin \
    out/kernel_entry.o out/kernel.o \
    --oformat binary

# Pad kernel to exactly 100 sectors (50 KiB)
truncate -s $((100 * 512)) out/kernel.bin

# Create disk image
cat out/boot.bin out/kernel.bin > out/os-image.bin

qemu-system-x86_64 -drive format=raw,file=out/os-image.bin
