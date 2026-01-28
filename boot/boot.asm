bits 16
org 0x7C00

_start:
    ; save boot drive number
    mov [boot_drive], dl

    ; init segement registers to 0
    xor ax, ax
    mov ds, ax
    mov es, ax

    ; Setup stack
    mov ss, ax
    mov sp, 0x7C00

    mov si, msg
    call print_string

    ; Load kernel using LBA
    mov word [dap_segment], 0x0000
    mov word [dap_offset], 0x8000
    mov word [dap_sectors], 100 ; Load 100 sectors (50KB)
    mov dword [dap_lba_low], 1 ; Start from LBA 1 (sector after bootloader)
    mov dword [dap_lba_high], 0

    call disk_load_lba

    ; Print sucess message
    mov si, msg_loaded
    call print_string

    ; jmp to loaded kernel
    jmp 0x0000:0x8000

    ; infinite loop
    jmp _loop

; ===============================================
; Load sectors using LBA
; Uses the Disk Address Packet (DAP) structure
; Input: DAP structure must be filled
; ===============================================
disk_load_lba:
    pusha

    mov ah, 0x42 ; Extended Read Function
    mov dl, [boot_drive]
    mov si, disk_address_packet
    int 0x13

    jc .disk_error

    ;restore DS
    xor ax, ax
    mov ds, ax

    popa
    ret

.disk_error:
    mov si, msg_disk_error
    call print_string
    ret

align 4
disk_address_packet:
    db 0x10             ; Size of DAP (16 bytes)
    db 0                ; Always 0
dap_sectors:
    dw 0                ; Number of sectors to read
dap_offset:
    dw 0                ; Memory offset
dap_segment:
    dw 0                ; Memory segment
dap_lba_low:
    dd 0                ; Lower 32 bits of LBA
dap_lba_high:
    dd 0                ; Upper 32 bits of LBA

%include "boot/print.asm"

msg: db "Booting jackcatos!", 0x0D, 0x0A, 0
msg_loaded: db "Kernel loaded!", 0x0D, 0x0A, 0
msg_disk_error: db "Failed to load Kernel!", 0x0D, 0x0A, 0
boot_drive: db 0

_loop:
    jmp _loop

times 510-($-$$) db 0
db 0x55
db 0xAA