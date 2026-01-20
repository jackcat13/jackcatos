bits 16
org 0x1000

kernel_start:
    mov si, msg_kernel_started
    call print_string
    jmp _loop

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

msg_kernel_started: db "Welcome in the kernel!", 0x0D, 0x0A, 0

_loop:
    jmp _loop

; Pad to fill sector
times 512-($-$$) db 0
