; kernel entry point

VGA_BUFFER_ADDR equ 0xB8000    ; physical address of the VGA text buffer
GREEN_OK        equ 0x2F4B2F4F ; bytes which will be written to the VGA buffer
MB_MAGIC        equ 0x36D76289 ; multiboot2 magic number which should be found in eax
CPUID_IMPLICIT  equ 0x80000000 ; implicit argument for cpuid, will allow us to determine largest supported argument
EXT_PROC_INFO   equ 0x80000001 ; minimum argument needed for extended processor information from cpuid

global start
extern lm_start

section .text
bits 32
start:
    mov esp, stack_top  ; set up the stack pointer so we can make function calls

    call check_mb       ; check that we were indeed loaded by a multiboot2 bootloader
    call check_lm       ; check for long mode availability (64 bit mode)
    call set_up_page_tables
    call enable_paging
    call enable_sse

    ; load the 64 bit global descriptor table (GDT)
    lgdt [gdt64.pointer]

    ; update selectors
    mov ax, gdt64.data
    mov ss, ax ; stack selector
    mov ds, ax ; data selector
    mov es, ax ; extra selector

    jmp gdt64.code:lm_start ; jump away to long mode, never to return

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

set_up_page_tables:
    ; map first P4 entry to P3 table
    mov eax, p3_table
    or eax, 0b11 ; present + writable
    mov [p4_table], eax

    ; map first P3 entry to P2 table
    mov eax, p2_table
    or eax, 0b11 ; present + writable
    mov [p3_table], eax

    ; map each P2 entry to a huge 2 MiB page
    mov ecx, 0

.map_p2_table:
    ; map ecx-th P2 entry to a huge page that starts at address 2 MiB * ecx
    mov eax, 1 << 21 ; == 2 MiB
    mul ecx
    or eax, 0b10000011 ; present + writable + huge
    mov [p2_table + ecx * 8], eax ; map the ecx-th entry

    inc ecx
    cmp ecx, 512 ; if ecx is 512, then then whole P2 table is mapped
    jne .map_p2_table

    ret

enable_paging:
    ; load P4 table to cr3 register (the cpu uses this to access the P4 table)
    mov eax, p4_table
    mov cr3, eax

    ; enable PAE-flag in cr4 (physical address extension)
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; set the long mode bit in the EFER MSR (model specific register)
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; enable paging in the cr0 register
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ret

enable_sse:
    ; check for SSE availability
    mov eax, 0x1
    cpuid
    test edx, 1 << 25
    jz .no_sse

    ; enable SSE
    mov eax, cr0
    and ax, 0xFFFB      ; clear coprocessor emulation CR0.EM
    or ax, 0x2          ; set coprocessor monitoring  CR0.MP
    mov cr0, eax
    mov eax, cr4
    or ax, 3 << 9       ; set CR4.OSFXSR and CR4.OSXMMEXCPT
    mov cr4, eax

    ret
.no_sse:
    mov al, "a"
    jmp error

section .bss
align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096
stack_bottom:
    resb 64
stack_top:

section .rodata
gdt64:
    dq 0 ; zero entry
.code: equ $ - gdt64
    dq (1 << 44) | (1 << 47) | (1 << 41) | (1 << 43) | (1 << 53) ; code segment
.data: equ $ - gdt64
    dq (1 << 44) | (1 << 47) | (1 << 41) ; data segment
.pointer:
    dw $ - gdt64 - 1
    dq gdt64
