bits 16
org 0x7C00

_start:
    ; save boot drive number
    mov [boot_drive], dl

    ; init segement registers to 0
    xor ax, ax
    mov ds, ax
    mov es, ax

    mov si, msg
    call print_string

    ; load kernel from disk
    mov bx, 0x1000 ; Memory address where to load data
    mov dh, 1 ; Number of sectors
    mov dl, [boot_drive] ; Drive number
    call disk_load

    ; Print sucess message
    mov si, msg_loaded
    call print_string

    ; jmp to loaded kernel
    jmp 0x0000:0x1000

    ; infinite loop
    jmp _loop

disk_load:
    push dx ; Save number of sectors to read

    mov ah, 0x02 ; read sector function
    mov al, dh ; number of sectors to read
    mov ch, 0 ; cylinder / track number
    mov cl, 2 ; sector 2 since sector 1 is bootloader
    mov dh, 0 ; head number
    int 0x13 ;
    jc .disk_error

    pop dx ; restore dx
    cmp al, dh ; Check if we read correct number of sectors
    jne .disk_error
    ret

.disk_error:
    mov si, msg_disk_error
    call print_string
    ret

print_string:
    lodsb
    cmp al, 0
    je .done

    ; call video interruption to print to screen
    mov ah, 0x0E
    mov bh, 0
    int 0x10
    jmp print_string

.done:
    ret

msg: db "Booting jackcatos!", 0x0D, 0x0A, 0
msg_loaded: db "Kernel loaded!", 0x0D, 0x0A, 0
msg_disk_error: db "Failed to load Kernel!", 0x0D, 0x0A, 0
boot_drive: db 0

_loop:
    jmp _loop

times 510-($-$$) db 0
db 0x55
db 0xAA