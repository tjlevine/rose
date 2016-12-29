; kernel entry point

VGA_BUFFER_ADDR equ 0xB8000    ; physical address of the VGA text buffer
GREEN_OK        equ 0x2F4B2F4F ; bytes which will be written to the VGA buffer
MB_MAGIC        equ 0x36D76289 ; multiboot2 magic number which should be found in eax
CPUID_IMPLICIT  equ 0x80000000 ; implicit argument for cpuid, will allow us to determine largest supported argument
EXT_PROC_INFO   equ 0x80000001 ; minimum argument needed for extended processor information from cpuid

global start

section .text
bits 32
start:
    mov esp, stack_top  ; set up the stack pointer so we can make function calls

    call check_mb       ; check that we were indeed loaded by a multiboot2 bootloader
    call check_lm       ; check for long mode availability (64 bit mode)

    ; print 'OK' to the screen
    mov dword [VGA_BUFFER_ADDR], GREEN_OK
    hlt

error:
    ; write "ERR: X", where X is an ASCII character in AL, to the screen
    ; with white text on a red background
    mov dword [VGA_BUFFER_ADDR + 0x0], 0x4F524F45
    mov dword [VGA_BUFFER_ADDR + 0x4], 0x4F3A4F52
    mov dword [VGA_BUFFER_ADDR + 0x8], 0x4F204F20
    mov byte  [VGA_BUFFER_ADDR + 0xA], al
    hlt

check_mb:
    cmp eax, MB_MAGIC   ; the MB_MAGIC sequence is required to be writted to eax by
                        ; the multiboot2 bootloader just before giving control to the kernel
    jne .no_mb
    ret
.no_mb:
    mov al, "0"
    jmp error

check_cpuid:
    ; cpuid instruction may not be supported, so we need to probe for its existence
    ; we can do that by attempting to flip the ID bit (bit 21) in the FLAGS register
    ; if we are able to flip it, and it stays flipped, then cpuid must be supported

    ; copy FLAGS to eax
    pushfd
    pop eax

    ; save a copy in ecx for later
    mov ecx, eax

    ; flip bit 21 (ID bit)
    xor eax, 1 << 21

    ; copy eax back to FLAGS
    push eax
    popfd

    ; copy FLAGS back to eax
    pushfd
    pop eax

    ; restore the old value of FLAGS stored in ecx
    push ecx
    popfd

    ; compare ecx and eax to see if bit 21 really flipped
    cmp eax, ecx
    je .no_cpuid
    ret
.no_cpuid:
    mov al, "1"
    jmp error

check_lm:
    ; to check if long mode is available on this processor, we need to get
    ; the extended processor info from the cpuid instruction, using argument
    ; EXT_PROC_INFO. However, EXT_PROC_INFO may not be available from cpuid,
    ; so we need to make sure it is by using the CPUID_IMPLICIT argument to
    ; find the largest supported cpuid argument. If this maximum argument is
    ; at least EXT_PROC_INFO, then we can check if long mode is available by
    ; checking bit 29 (LM bit) in edx.

    call check_cpuid    ; check for availability of the cpuid instruction
    call check_ext_proc ; check for availability of extended processor info from cpuid

    ; extended processor info is available
    mov eax, EXT_PROC_INFO  ; load cpuid extended processor info argument
    cpuid                   ; loads feature bits into ecx and edx
    test edx, 1 << 29       ; check bit 29 (LM bit) in edx
    jz .no_lm               ; if bit 29 is not set, long mode is not available
    ret
.no_lm:
    mov al, "3"
    jmp error

check_ext_proc:
    mov eax, CPUID_IMPLICIT ; load cpuid implicit argument
    cpuid                   ; get highest supported cpuid argument
    cmp eax, EXT_PROC_INFO  ; it must be at least EXT_PROC_INFO
    jb .no_ext_proc         ; if it's less, EXT_PROC_INFO is not supported
    ret
.no_ext_proc:
    mov al, "2"
    jmp error

section .bss
stack_bottom:
    resb 64
stack_top:
