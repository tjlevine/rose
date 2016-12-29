; 64-bit (long mode) entry point
; all code in this file is executed in a long mode context

global lm_start

OKAY equ 0x2F592F412F4B2F4F    ; bytes to be written to the text buffer
VGA_BUFFER_ADDR equ 0xB8000    ; physical address of the VGA text buffer

section .text
bits 64
lm_start:
    ; call the rust entry point
    extern rust_main
    call rust_main

.os_returned:
    ; rust main returned, print `OS returned!`
    mov rax, 0x4F724F204F534F4F
    mov [VGA_BUFFER_ADDR + 0x00], rax
    mov rax, 0x4F724F754F744F65
    mov [VGA_BUFFER_ADDR + 0x08], rax
    mov rax, 0x4F214F644F654F6E
    mov [VGA_BUFFER_ADDR + 0x10], rax
    hlt
