bits 16
org 0x1000

kernel_start:
    mov si, msg_kernel_started
    call print_string

    mov si, msg_switching_pm
    call print_string

    ; Switch to protected mode
    cli ; Disable interrupts
    lgdt [gdt_descriptor] ; Load GDT

    mov eax, cr0 ; Get current CR0
    or eax, 0x1 ; Set PE (Protection Enable)
    mov cr0, eax ; Entering protected mode

    jmp CODE_SEG:init_pm ; Far jump to 32-bit code

align 8
%include "boot/gdt.asm"
%include "boot/print.asm"

bits 32
init_pm:
    ; Set up segment registers for protected mode
    mov ax, DATA_SEG
    mov ds, ax
    mov ss, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; Set ip stack
    mov ebp, 0x90000
    mov esp, ebp

    ; Print success message on the 5th line to avoid overwriting BIOS messages
    mov ebx, msg_pm_success     ; Point EBX to the string
    mov edx, 0xB8780            ; Point EDX to the 13th line of VGA memory
    call print_string_pm

    jmp $                       ; Hang

; Print string in 32-bit protected mode
; Params:
; EBX = Address of the null-terminated string
; EDX = Video memory address to write to
print_string_pm:
    pusha

.loop:
    mov al, [ebx]               ; Get character
    mov ah, 0x0F                ; White on black background

    cmp al, 0                   ; Check for null terminator
    je .done

    mov [edx], ax               ; Write to VGA memory
    add ebx, 1                  ; Next character
    add edx, 2                  ; Next VGA position (char + attribute)

    jmp .loop

.done:
    popa
    ret

; Messages
msg_kernel_started: db "Kernel started in 16-bit mode", 0x0D, 0x0A, 0
msg_switching_pm: db "Switching to 32-bit protected mode...", 0x0D, 0x0A, 0
msg_pm_success: db "32-bit Protected Mode Active!", 0

; Pad to fill sector
times 7680-($-$$) db 0
