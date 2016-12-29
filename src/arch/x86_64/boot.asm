; kernel entry point

VGA_BUFFER_ADDR equ 0xB8000    ; physical address of the VGA text buffer
GREEN_OK        equ 0x2F4B2F4F ; bytes which will be written to the VGA buffer

global start

section .text
bits 32
start:
    ; print 'OK' to the screen
    mov dword [VGA_BUFFER_ADDR], GREEN_OK
    hlt
