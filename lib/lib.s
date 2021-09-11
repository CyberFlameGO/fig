section .text
global put_int
put_int:
        sub     rsp, 40
        xor     r10d, r10d
        mov     BYTE [rsp+31], 10
        test    rdi, rdi
        jns     .L2
        neg     rdi
        mov     r10d, 1
.L2:
        mov     r8d, 1
        lea     r9, [rsp+31]
        mov     rsi, -3689348814741910323
.L3:
        mov     rax, rdi
        mov     rcx, r9
        mul     rsi
        sub     rcx, r8
        shr     rdx, 3
        lea     rax, [rdx+rdx*4]
        add     rax, rax
        sub     rdi, rax
        mov     rax, r8
        add     r8, 1
        add     edi, 48
        mov     BYTE [rcx], dil
        mov     rdi, rdx
        test    rdx, rdx
        jne     .L3
        test    r10d, r10d
        je      .L4
        not     r8
        mov     BYTE [rsp+32+r8], 45
        lea     r8, [rax+2]
.L4:
        mov     eax, 32
        mov     rdx, r8
        mov     edi, 1
        sub     rax, r8
        lea     rsi, [rsp+rax]
        mov     rax, 1
        syscall
        add     rsp, 40
        ret
