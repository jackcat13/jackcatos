; Prints any string present in si register
; Params:
; si = Address of the null-terminated string
print_string:
    lodsb
    cmp al, 0
    je .done

    mov ah, 0x0E
    mov bh, 0
    int 0x10
    jmp print_string

.done:
    ret