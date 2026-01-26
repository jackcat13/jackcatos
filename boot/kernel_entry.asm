global _start
extern kernel_main

section .text
bits 16

_start:
    mov si, msg_kernel_started
    call print_string

    mov si, msg_switching_pm
    call print_string

    ; --- VESA VBE GRAPHICS SETUP ---
    ; 1. Get VBE Mode Info to find the Framebuffer Address
    mov ax, 0x4F01 ; Function: Get Mode Info
    mov cx, 0x4118 ; Mode: 1024x768 x 24-bit color (Linear Framebuffer bit set)
    mov di, 0x5000 ; Target address to store Info Block (0x5000)
    int 0x10 ; Call BIOS

    cmp ax, 0x004F ; Check for success
    jne .vbe_error

    ; 2. Set the Mode
    mov ax, 0x4F02 ; Function: Set VBE Mode
    mov bx, 0x4118 ; Mode + Linear Framebuffer bit (0x4000)
    int 0x10

    ; Switch to protected mode
    cli ; Disable interrupts
    lgdt [gdt_descriptor] ; Load GDT

    mov eax, cr0 ; Get current CR0
    or eax, 0x1 ; Set PE (Protection Enable)
    mov cr0, eax ; Entering protected mode

    jmp CODE_SEG:init_pm ; Far jump to 32-bit code

.vbe_error:
    jmp $

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

    call check_long_mode
    call setup_page_tables

    ; Enable SSE (needed by Rust for floating point (xmm registers)
    mov eax, cr0
    and ax, 0xFFFB ; Clear EM (Emulation)
    or ax, 0x2 ; Set MP (Monitor Coprocessor)
    mov cr0, eax
    mov eax, cr4
    or eax, (1 << 9)    ; Set OSFXSR (OS support for FXSAVE/FXRSTOR)
    or eax, (1 << 10)   ; Set OSXMMEXCPT (OS support for unmasked SIMD exceptions)
    mov cr4, eax

    ; Enable PAE
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; Enable Long Mode (EFER MSR)
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; Enable Paging (CR0)
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    jmp CODE_SEG_64:init_lm

%include "boot/cpuid.asm"
%include "boot/paging.asm"

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

bits 64
init_lm:
    mov ax, DATA_SEG
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    ; Print "64-bit Long Mode Active!"
    ; We'll print it on line 15 (offset 0xB8960) to avoid overwriting previous messages
    mov rbx, msg_lm_success     ; Address of string
    mov rdx, 0xB8960            ; Address in VGA buffer (0xB8000 + 15 * 160)
    call print_string_lm

    ; Entry point written in Rust
    call kernel_main

    hlt
    jmp $

; Print string in 64-bit Long Mode
; Params:
; RBX = Address of the null-terminated string
; RDX = Video memory address to write to
print_string_lm:
    ; pusha doesn't exist in 64-bit mode, so we save specific registers
    push rax
    push rbx
    push rdx

.loop:
    mov al, [rbx]               ; Get character
    mov ah, 0x0F                ; White on black
    cmp al, 0                   ; Check for null terminator
    je .done

    mov [rdx], ax               ; Write to VGA memory
    inc rbx                     ; Next char
    add rdx, 2                  ; Next VGA position
    jmp .loop

.done:
    ; Restore registers in reverse order
    pop rdx
    pop rbx
    pop rax
    ret

; Messages
msg_kernel_started: db "Kernel started in 16-bit mode", 0x0D, 0x0A, 0
msg_switching_pm: db "Switching to 32-bit protected mode...", 0x0D, 0x0A, 0
msg_pm_success: db "32-bit Protected Mode Active!", 0
msg_lm_success: db "64-bit Long Mode Active!", 0