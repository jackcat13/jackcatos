nasm -f bin boot/boot.asm -o boot.bin
nasm -f elf64 boot/kernel_entry.asm -o kernel_entry.o

cargo rustc --release -- --emit obj=kernel.o

ld.lld -T linker.ld -o kernel.bin kernel_entry.o kernel.o --oformat binary

dd if=/dev/zero bs=512 count=15 >> kernel.bin

cat boot.bin kernel.bin > os-image.bin

qemu-system-x86_64 -drive format=raw,file=os-image.bin