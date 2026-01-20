; Temporary kernel entry point (sector 2)
; This will be replaced by Rust kernel later

jmp $   ; Just hang for now

; Pad to fill sector
times 512-($-$$) db 0
