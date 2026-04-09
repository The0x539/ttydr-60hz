patch_names:
	.asciz "timekeeper_update"
	.asciz "yuu_03"
	.asciz "ac_gauge"
	.asciz "graphics_init"
	.byte 0
	.align 4

graphics_init:
	.int 0x309780
	.int 0x309990

	.int 0x334
	movk x9, #2, lsl #32
	movk x9, #1, lsl #32

	.int -1

timekeeper_update:
	.int 0x2e7540
	.int 0x2e7750

	.int 0x54
	mov w9, #2
	mov w9, #1

	.int 0x7c
	cmp w8, #2
	cmp w8, #1

	.int 0x90
	fmov s0, #2.0
	fmov s0, #1.0

	.int -1

yuu_03:
	.int 0xa25520
	.int 0xa25f10

	.macro sixty offset, reg
		.int \offset
		mov \reg, #0x42700000 // 60.0
		mov \reg, #0x42f00000 // 120.0
	.endm

	sixty 0x11b8, w8
	sixty 0x15a4, w8
	sixty 0x1aa8, w9
	sixty 0x1cec, w8
	sixty 0x206c, w9
	sixty 0x22a0, w9

	.int -1

ac_gauge:
	.int 0x471250
	.int 0x471480

	.int 0x3e8
	movk w10, #0x3ecc, lsl #16
	movk w10, #0x3f4c, lsl #16

	.int 0x408
	fminnm s3, s0, s1
	fmov s3, s0

	.int 0x460
	fmul s7, s3, s4
	fmov s7, s4

	.int -1
