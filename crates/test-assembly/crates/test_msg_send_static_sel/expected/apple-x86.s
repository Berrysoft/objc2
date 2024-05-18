	.section	__TEXT,__text,regular,pure_instructions
	.intel_syntax noprefix
	.globl	_handle_with_sel
	.p2align	4, 0x90
_handle_with_sel:
	push	ebp
	mov	ebp, esp
	sub	esp, 8
	call	L0$pb
L0$pb:
	pop	eax
	sub	esp, 8
	push	dword ptr [eax + L_OBJC_SELECTOR_REFERENCES_ac93c21013e05a7d-L0$pb]
	push	dword ptr [ebp + 8]
	call	_objc_msgSend
	add	esp, 24
	pop	ebp
	ret

	.globl	_handle_alloc_init
	.p2align	4, 0x90
_handle_alloc_init:
	push	ebp
	mov	ebp, esp
	push	ebx
	push	edi
	push	esi
	sub	esp, 12
	call	L1$pb
L1$pb:
	pop	edi
	sub	esp, 12
	push	dword ptr [ebp + 8]
	call	_objc_alloc
	add	esp, 16
	mov	esi, eax
	mov	eax, dword ptr [edi + LL_OBJC_SELECTOR_REFERENCES_init$non_lazy_ptr-L1$pb]
	mov	ebx, dword ptr [eax]
	sub	esp, 8
	push	ebx
	push	esi
	call	_objc_msgSend
	add	esp, 16
	test	eax, eax
	je	LBB1_2
	add	esp, 12
	pop	esi
	pop	edi
	pop	ebx
	pop	ebp
	ret
LBB1_2:
	sub	esp, 4
	lea	eax, [edi + l_anon.[ID].1-L1$pb]
	push	eax
	push	ebx
	push	esi
	call	SYM(<objc2::__macro_helpers::method_family::RetainSemantics<3_u8> as objc2::__macro_helpers::msg_send_id::MsgSendIdFailed>::failed::GENERATED_ID, 0)

	.globl	_use_generic
	.p2align	4, 0x90
_use_generic:
	push	ebp
	mov	ebp, esp
	push	edi
	push	esi
	call	L2$pb
L2$pb:
	pop	esi
	mov	edi, dword ptr [ebp + 8]
	sub	esp, 4
	push	dword ptr [esi + L_OBJC_SELECTOR_REFERENCES_8071e784bfc1c0df-L2$pb]
	push	dword ptr [esi + L_OBJC_SELECTOR_REFERENCES_df2cc6f80edda422-L2$pb]
	push	edi
	call	_objc_msgSend
	add	esp, 12
	push	dword ptr [esi + L_OBJC_SELECTOR_REFERENCES_8071e784bfc1c0df-L2$pb]
	push	dword ptr [esi + L_OBJC_SELECTOR_REFERENCES_e520bf80aade9209-L2$pb]
	push	edi
	call	_objc_msgSend
	add	esp, 12
	push	dword ptr [esi + L_OBJC_SELECTOR_REFERENCES_8071e784bfc1c0df-L2$pb]
	push	dword ptr [esi + L_OBJC_SELECTOR_REFERENCES_bab7ff70ff0c78f0-L2$pb]
	push	edi
	call	_objc_msgSend
	add	esp, 16
	pop	esi
	pop	edi
	pop	ebp
	ret

	.section	__TEXT,__const
l_anon.[ID].0:
	.ascii	"crates/$DIR/lib.rs"

	.section	__DATA,__const
	.p2align	2, 0x0
l_anon.[ID].1:
	.long	l_anon.[ID].0
	.asciz	";\000\000\000\016\000\000\000\005\000\000"

	.section	__TEXT,__objc_methname,cstring_literals
	.globl	L_OBJC_METH_VAR_NAME_ac93c21013e05a7d
L_OBJC_METH_VAR_NAME_ac93c21013e05a7d:
	.asciz	"someSelector"

	.section	__DATA,__objc_selrefs,literal_pointers,no_dead_strip
	.globl	L_OBJC_SELECTOR_REFERENCES_ac93c21013e05a7d
	.p2align	2, 0x0
L_OBJC_SELECTOR_REFERENCES_ac93c21013e05a7d:
	.long	L_OBJC_METH_VAR_NAME_ac93c21013e05a7d

	.section	__DATA,__objc_imageinfo,regular,no_dead_strip
	.globl	L_OBJC_IMAGE_INFO_ac93c21013e05a7d
	.p2align	2, 0x0
L_OBJC_IMAGE_INFO_ac93c21013e05a7d:
	.asciz	"\000\000\000\000@\000\000"

	.section	__TEXT,__objc_methname,cstring_literals
	.globl	L_OBJC_METH_VAR_NAME_8071e784bfc1c0df
L_OBJC_METH_VAR_NAME_8071e784bfc1c0df:
	.asciz	"generic:selector:"

	.section	__DATA,__objc_selrefs,literal_pointers,no_dead_strip
	.globl	L_OBJC_SELECTOR_REFERENCES_8071e784bfc1c0df
	.p2align	2, 0x0
L_OBJC_SELECTOR_REFERENCES_8071e784bfc1c0df:
	.long	L_OBJC_METH_VAR_NAME_8071e784bfc1c0df

	.section	__DATA,__objc_imageinfo,regular,no_dead_strip
	.globl	L_OBJC_IMAGE_INFO_8071e784bfc1c0df
	.p2align	2, 0x0
L_OBJC_IMAGE_INFO_8071e784bfc1c0df:
	.asciz	"\000\000\000\000@\000\000"

	.section	__TEXT,__objc_methname,cstring_literals
	.globl	L_OBJC_METH_VAR_NAME_df2cc6f80edda422
L_OBJC_METH_VAR_NAME_df2cc6f80edda422:
	.asciz	"performSelector:"

	.section	__DATA,__objc_selrefs,literal_pointers,no_dead_strip
	.globl	L_OBJC_SELECTOR_REFERENCES_df2cc6f80edda422
	.p2align	2, 0x0
L_OBJC_SELECTOR_REFERENCES_df2cc6f80edda422:
	.long	L_OBJC_METH_VAR_NAME_df2cc6f80edda422

	.section	__DATA,__objc_imageinfo,regular,no_dead_strip
	.globl	L_OBJC_IMAGE_INFO_df2cc6f80edda422
	.p2align	2, 0x0
L_OBJC_IMAGE_INFO_df2cc6f80edda422:
	.asciz	"\000\000\000\000@\000\000"

	.section	__TEXT,__objc_methname,cstring_literals
	.globl	L_OBJC_METH_VAR_NAME_e520bf80aade9209
L_OBJC_METH_VAR_NAME_e520bf80aade9209:
	.asciz	"performSelector:"

	.section	__DATA,__objc_selrefs,literal_pointers,no_dead_strip
	.globl	L_OBJC_SELECTOR_REFERENCES_e520bf80aade9209
	.p2align	2, 0x0
L_OBJC_SELECTOR_REFERENCES_e520bf80aade9209:
	.long	L_OBJC_METH_VAR_NAME_e520bf80aade9209

	.section	__DATA,__objc_imageinfo,regular,no_dead_strip
	.globl	L_OBJC_IMAGE_INFO_e520bf80aade9209
	.p2align	2, 0x0
L_OBJC_IMAGE_INFO_e520bf80aade9209:
	.asciz	"\000\000\000\000@\000\000"

	.section	__TEXT,__objc_methname,cstring_literals
	.globl	L_OBJC_METH_VAR_NAME_bab7ff70ff0c78f0
L_OBJC_METH_VAR_NAME_bab7ff70ff0c78f0:
	.asciz	"performSelector:"

	.section	__DATA,__objc_selrefs,literal_pointers,no_dead_strip
	.globl	L_OBJC_SELECTOR_REFERENCES_bab7ff70ff0c78f0
	.p2align	2, 0x0
L_OBJC_SELECTOR_REFERENCES_bab7ff70ff0c78f0:
	.long	L_OBJC_METH_VAR_NAME_bab7ff70ff0c78f0

	.section	__DATA,__objc_imageinfo,regular,no_dead_strip
	.globl	L_OBJC_IMAGE_INFO_bab7ff70ff0c78f0
	.p2align	2, 0x0
L_OBJC_IMAGE_INFO_bab7ff70ff0c78f0:
	.asciz	"\000\000\000\000@\000\000"

	.section	__IMPORT,__pointers,non_lazy_symbol_pointers
LL_OBJC_SELECTOR_REFERENCES_init$non_lazy_ptr:
	.indirect_symbol	L_OBJC_SELECTOR_REFERENCES_init
	.long	0

.subsections_via_symbols
