/* SPDX-License-Identifier: GPL-2.0-only */
/* AArch64 PID1: no libc, mounts, files, USB or probe. Return to bootloader.
 * If reboot fails, exiting PID1 triggers panic; image sets panic=5. */
.section .text
.global _start
.type _start, %function
_start:
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
.size _start, .-_start
.section .rodata
bootloader: .asciz "bootloader"
.section .note.GNU-stack,"",%progbits
