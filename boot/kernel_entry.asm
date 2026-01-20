bits 16
org 0x1000

kernel_start:
    mov si, msg_kernel_started
    call print_string
    jmp _loop

%include "boot/print.asm"

msg_kernel_started: db "Welcome in the kernel!", 0x0D, 0x0A, 0

_loop:
    jmp _loop

; Pad to fill sector
times 512-($-$$) db 0
