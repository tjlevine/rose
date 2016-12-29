; Multiboot header
; This gets loaded by any multiboot 2 combatible bootloader, eg. grub

MAGIC equ 0xE85250D6
ARCH  equ 0
TYPE  equ 0
FLAGS equ 0
SIZE  equ 8


section .multiboot_header
header_start:
    dd MAGIC                     ; magic number (multiboot 2)
    dd ARCH                      ; architecture 0 (protected mode i386)
    dd header_end - header_start ; header length

    ; checksum
    dd 0x100000000 - (MAGIC + ARCH + (header_end - header_start))

    ; insert optional multiboot tags here

    ; required end tags
    dw TYPE
    dw FLAGS
    dd SIZE
header_end:

