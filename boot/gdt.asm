; GDT - Global Descriptor Table for 32-bit Protected Mode

gdt_start:
    ; Null descriptor (required - 8 bytes of zeros)
    dq 0x0

gdt_code:
    ; Code segment descriptor
    dw 0xFFFF       ; Limit (bits 0-15) - segment can access 0xFFFFF
    dw 0x0          ; Base (bits 0-15) - starts at 0
    db 0x0          ; Base (bits 16-23)
    db 10011010b    ; Access byte: present=1, privilege=00, type=1, code=1, conforming=0, readable=1, accessed=0
    db 11001111b    ; Flags: granularity=1(4KB), 32-bit=1, 64-bit=0, AVL=0 + Limit(bits 16-19)=1111
    db 0x0          ; Base (bits 24-31)

gdt_data:
    ; Data segment descriptor
    dw 0xFFFF       ; Limit (bits 0-15)
    dw 0x0          ; Base (bits 0-15)
    db 0x0          ; Base (bits 16-23)
    db 10010010b    ; Access byte: present=1, privilege=00, type=1, code=0, expand_down=0, writable=1, accessed=0
    db 11001111b    ; Flags (same as code segment)
    db 0x0          ; Base (bits 24-31)

gdt_code_64:
    ; 64-bit Code segment descriptor
    dw 0xFFFF       ; Limit (ignored in 64-bit)
    dw 0x0          ; Base (ignored in 64-bit)
    db 0x0          ; Base
    db 10011010b    ; Access: present, ring 0, code, executable, readable
    db 10101111b    ; Flags: Long Mode (L)=1, 32-bit (D)=0
    db 0x0          ; Base

gdt_end:

gdt_descriptor:
    dw gdt_end - gdt_start - 1    ; Size of GDT (16-bit)
    dd gdt_start                   ; Address of GDT (32-bit)

; Define constants for segment selectors
CODE_SEG equ gdt_code - gdt_start    ; 0x08
DATA_SEG equ gdt_data - gdt_start    ; 0x10
CODE_SEG_64 equ gdt_code_64 - gdt_start ; 0x18