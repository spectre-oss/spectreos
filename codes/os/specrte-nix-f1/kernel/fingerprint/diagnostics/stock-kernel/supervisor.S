/* SPDX-License-Identifier: GPL-2.0-only */
/* PID1 uses only syscalls. Run /diagnostic as a child, then return to
 * bootloader on child exit or after at most 90 one-second waits. */
.section .text
.global _start
.type _start,%function
_start:
 mov x0, #17 /* clone(SIGCHLD), fork-like */
 mov x1, #0
 mov x2, #0
 mov x3, #0
 mov x4, #0
 mov x8, #220
 svc #0
 cbz x0, child
 tbnz x0, #63, reboot
 mov x20, x0
 mov x19, #90
wait_child:
 mov x0, x20
 mov x1, #0
 mov x2, #1 /* WNOHANG */
 mov x3, #0
 mov x8, #260 /* wait4 */
 svc #0
 cbnz x0, reboot
 adr x0, delay
 mov x1, #0
 mov x8, #101 /* nanosleep */
 svc #0
 subs x19, x19, #1
 b.ne wait_child
reboot:
 movz w0, #0xdead
 movk w0, #0xfee1, lsl #16
 movz w1, #0x1969
 movk w1, #0x2812, lsl #16
 movz w2, #0xc3d4
 movk w2, #0xa1b2, lsl #16
 adr x3, bootloader
 mov x8, #142
 svc #0
 mov x0, #1
 mov x8, #94
 svc #0
1: b 1b
child:
 adr x0, diagnostic
 adr x1, argv
 adr x2, envp
 mov x8, #221 /* execve */
 svc #0
 mov x0, #127
 mov x8, #94
 svc #0
2: b 2b
.size _start, .-_start
.section .rodata
bootloader: .asciz "bootloader"
diagnostic: .asciz "/diagnostic"
.balign 8
argv: .quad diagnostic, 0
envp: .quad 0
delay: .quad 1, 0
.section .note.GNU-stack,"",%progbits
