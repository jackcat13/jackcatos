setup_page_tables:
    ; We'll use 0x10000 for page tables (well above our kernel at 0x1000)
    mov edi, 0x10000
    mov cr3, edi

    xor eax, eax
    mov ecx, 4096
    rep stosd

    mov edi, cr3

    ; PML4[0] -> PDPT
    mov dword [edi], 0x11003    ; 0x11000 | Present | Writable
    add edi, 0x1000

    ; PDPT[0] -> PD
    mov dword [edi], 0x12003    ; 0x12000 | Present | Writable
    add edi, 0x1000

    ; PD[0] -> 2MB Page (Identity map 0-2MB)
    mov dword [edi], 0x83       ; 0x0 | Present | Writable | Huge Page

    ret