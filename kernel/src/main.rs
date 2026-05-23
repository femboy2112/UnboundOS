// UnboundOS kernel entry point.
//
// This is the bare-metal entry. It MUST follow spec section 3.2 strictly:
// disable interrupts, init serial, emit boot heartbeat, validate
// bootloader handoff, install GDT/IDT, ingest memory map, init boot
// allocator, probe CPU features, enable permitted SIMD, init framebuffer
// (optional), init permanent kernel structures, load initial graph,
// enter orchestrator.
//
// Boot is never blind. Every path either reaches `UNBOUNDOS_BOOT_OK` on
// serial, or — when UART is unavailable — records into the boot
// diagnostic buffer and prints `BOOT_NO_SERIAL` /
// `BOOT_HEARTBEAT_BUFFER_PRESENT` once the framebuffer is up.

#![no_std]
#![no_main]
#![forbid(unsafe_op_in_unsafe_fn)]
#![feature(abi_x86_interrupt)]
// TODO M0/M1 (spec §13): drop this allow once the boot path, IDT,
// and arena allocator stop being stubs and the SimdTier variants
// are constructed by `cpu::detect_features`. The scaffolding types
// (ArenaId, AllocError, the non-Scalar SimdTier variants, the
// boot_diag snapshot reader, idt::is_installed, etc.) are declared
// ahead of their first use.
#![allow(dead_code)]
// Kernel idiom: bit-level descriptor pack/unpack relies on
// intentional truncating casts and lossy field widths. Pedantic
// floor stays denied for the rest of the workspace; this kernel
// crate carves out the cast lints.
#![allow(clippy::cast_possible_truncation, clippy::cast_lossless)]

use core::panic::PanicInfo;

core::arch::global_asm!(
    r#"
.section .multiboot2_header,"a"
.align 8
mb2_header_start:
    .long 0xe85250d6
    .long 0
    .long mb2_header_end - mb2_header_start
    .long -(0xe85250d6 + 0 + (mb2_header_end - mb2_header_start))
    .short 0
    .short 0
    .long 8
mb2_header_end:

.section .text.boot,"ax"
.global _mb2_start
.code32
_mb2_start:
    cli
    mov esp, offset boot_stack_top

    mov edi, offset boot_p2
    xor ecx, ecx
    xor eax, eax
1:
    mov edx, eax
    or edx, 0x83
    mov dword ptr [edi + ecx*8], edx
    mov dword ptr [edi + ecx*8 + 4], 0
    add eax, 0x200000
    inc ecx
    cmp ecx, 512
    jne 1b

    mov dword ptr [boot_p4], offset boot_p3 + 0x3
    mov dword ptr [boot_p4 + 4], 0
    mov dword ptr [boot_p3], offset boot_p2 + 0x3
    mov dword ptr [boot_p3 + 4], 0

    mov eax, cr4
    or eax, 0x20
    mov cr4, eax

    mov eax, offset boot_p4
    mov cr3, eax

    mov ecx, 0xc0000080
    rdmsr
    or eax, 0x100
    wrmsr

    mov eax, cr0
    or eax, 0x80000001
    mov cr0, eax

    lgdt [boot_gdtr]
    .byte 0xea
    .long long_mode_start
    .word 0x08

.code64
long_mode_start:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax
    mov rsp, offset boot_stack_top
    and rsp, -16
    call _start
2:
    hlt
    jmp 2b

.section .rodata.boot,"a"
.align 8
boot_gdt:
    .quad 0
    .quad 0x00af9a000000ffff
    .quad 0x00af92000000ffff
boot_gdt_end:
boot_gdtr:
    .word boot_gdt_end - boot_gdt - 1
    .quad boot_gdt

.section .bss.boot,"aw",@nobits
.align 4096
boot_p4:
    .skip 4096
boot_p3:
    .skip 4096
boot_p2:
    .skip 4096
.align 16
boot_stack_bottom:
    .skip 16384
boot_stack_top:
"#
);

mod arena;
mod boot;
mod boot_diag;
mod cpu;
mod framebuffer;
mod heartbeat;
mod idt;
mod operator_shell;
mod serial;
mod ssod;
mod storage;

/// Kernel entry point. Called by Limine after the CPU is in 64-bit long
/// mode. The handoff contract is defined in spec section 3.1.
///
/// # Safety
///
/// The bootloader has set up the CPU state per the Limine contract.
/// The function takes ownership of the CPU and never returns.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // SAFETY: this is the unique entry point. We hold exclusive access
    // to the CPU. Each substep is responsible for its own invariants;
    // see boot::run for the full sequence.
    unsafe { boot::run() }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // The panic path funnels into kernel_panic with a structured
    // diagnostic context. SSOD is the only legal exit from a panic.
    ssod::from_rust_panic(info)
}
