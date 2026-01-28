setup_page_tables:
    ; We'll use 0x12000 for page tables (well above our kernel at 0x1000)
    mov edi, 0x12000
    mov cr3, edi

    xor eax, eax
    mov ecx, 6144
    rep stosd

    mov eax, cr3
    mov edi, eax

    ; --- Level 4 (PML4) ---
    ; Map first entry to PDPT
    mov dword [edi], 0x13003      ; Point to PDPT at 0x13000 | Present | Writable

    ; --- Level 3 (PDPT) ---
    ; We need to map 4 entries (4GB total) to cover typical VBE Framebuffer locations
    ; PDPT[0] -> PD0 (0-1GB)
    ; PDPT[1] -> PD1 (1-2GB)
    ; PDPT[2] -> PD2 (2-3GB)
    ; PDPT[3] -> PD3 (3-4GB)

    mov eax, 0x14003 ; First PD at 0x14000
    mov dword [edi + 0x1000], eax

    add eax, 0x1000
    mov dword [edi + 0x1000 + 8], eax ; PDPT[1]

    add eax, 0x1000
    mov dword [edi + 0x1000 + 16], eax ; PDPT[2]

    add eax, 0x1000
    mov dword [edi + 0x1000 + 24], eax ; PDPT[3]

    ; --- Level 2 (Page Directories) ---
    ; We need to fill 4 Page Directories (2048 entries total)
    ; Each entry maps 2MB. 2048 * 2MB = 4GB.

    mov edi, 0x14000 ; Start of first PD
    mov eax, 0x83 ; Start at physical address 0 | Huge | Present | Writable
    mov ecx, 2048 ; 512 entries * 4 directories

.fill_pd:
    mov [edi], eax
    add eax, 0x200000 ; Add 2MB to physical address
    add edi, 8 ; Next entry
    loop .fill_pd

    ret
