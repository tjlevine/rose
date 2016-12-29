; 64-bit (long mode) entry point
; all code in this file is executed in a long mode context

global lm_start

OKAY equ 0x2F592F412F4B2F4F    ; bytes to be written to the text buffer
VGA_BUFFER_ADDR equ 0xB8000    ; physical address of the VGA text buffer

section .text
bits 64
lm_start:
    ; print OKAY to screen
    mov rax, OKAY
    mov qword [VGA_BUFFER_ADDR], rax
    hlt
