versions:
	.asciz "v1.0.0"
	.asciz "78f37bb55d015be3b368ec22af595455f1544dc1"
	.asciz "v1.0.1"
	.asciz "0effe4af1dec3a7966b934d4a7c3d2bf566a9c62"
	.byte 0
	.align 4

groups:
	.asciz "graphics_init"
	.asciz "timekeeper_update"
	.asciz "yuu_03"
	.asciz "ac_gauge"
	.asciz "ac_flee_gauge"
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
	movk w10, #0x3ecc, lsl #16 // the top half of 0.4_f32
	movk w10, #0x3f4c, lsl #16 // the top half of 0.8_f32

	.int 0x408
	fminnm s3, s0, s1
	fmov s3, s0

	.int 0x460
	fmul s7, s3, s4
	fmov s7, s4

	.int -1

ac_flee_gauge:
	.int 0x46d910
	.int 0x46db40

	.int 0x374
	movk w12, #0x3c23, lsl #16 // the top half of 0.01_f32
	movk w12, #0x3ca3, lsl #16 // the top half of 0.02_f32

	.int 0x38c
	fminnm s3, s0, s1
	fmov s3, s0

	.int 0x3b4
	fmul s4, s3, s7
	fmov s4, s7

	.int -1
