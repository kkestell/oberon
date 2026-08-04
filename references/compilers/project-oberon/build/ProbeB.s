.bss
.balign 8
ProbeB.p:
	.fill 8,1,0
/* end data */

.bss
.balign 4
ProbeB.calls:
	.fill 4,1,0
/* end data */

.text
.balign 16
ProbeB.Index:
	endbr64
	movl ProbeB.calls(%rip), %eax
	addl $1, %eax
	movl %eax, ProbeB.calls(%rip)
	movl $0, %eax
	ret
.type ProbeB.Index, @function
.size ProbeB.Index, .-ProbeB.Index
/* end function ProbeB.Index */

.text
.balign 16
ProbeB.Fold:
	endbr64
	pushq %rbp
	movq %rsp, %rbp
	movl $0, %esi
	movl $3, %edi
	callq oberon_out_int
	movl $0, %esi
	movl $3, %edi
	callq oberon_out_int
	movl ProbeB.calls(%rip), %edi
	movl $0, %esi
	callq oberon_out_int
	leave
	ret
.type ProbeB.Fold, @function
.size ProbeB.Fold, .-ProbeB.Fold
/* end function ProbeB.Fold */

.text
.balign 16
.ProbeB.init:
	endbr64
	pushq %rbp
	movq %rsp, %rbp
	callq ProbeB.Fold
	callq oberon_out_ln
	leave
	ret
.type .ProbeB.init, @function
.size .ProbeB.init, .-.ProbeB.init
/* end function .ProbeB.init */

.text
.balign 16
.globl main
main:
	endbr64
	pushq %rbp
	movq %rsp, %rbp
	callq oberon_init
	callq .ProbeB.init
	movl $0, %eax
	leave
	ret
.type main, @function
.size main, .-main
/* end function main */

.section .note.GNU-stack,"",@progbits
