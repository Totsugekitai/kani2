OUTPUT_FORMAT("elf64-x86-64");
ENTRY(kernel_main);

KERNEL_BASE = 0x100000;

SECTIONS {
    . = KERNEL_BASE;

    .text : {
        *(.text.main);
        *(.text.*);
    }

    .rodata : {
        *(.rodata);
        *(.rodata.*);

        . = ALIGN(4096);
        __cpu_local = .;
        KEEP(*(.cpu_local_head));
        KEEP(*(.cpu_local));
        __cpu_local_end = .;
        __cpu_local_size = __cpu_local_end - __cpu_local;
    }

    .data : {
        *(.data);
        *(.data.*);

        *(.got*);

        __bsp_cpu_local = .;
        . += __cpu_local_size;
        __bsp_cpu_local_end = .;
    }

    .bss : {
        __bss = .;
        *(.bss);
        *(.bss.*);
        __bss_end = .;

        /* The kernel page table (physical addresses). */
        . = ALIGN(4096);
        __kernel_pagetable_pml4 = .;
        . += 0x1000;
        __kernel_pagetable_pdpt = .;
        . += 0x1000;
        __kernel_pagetable_pd = .;
        . += 4 * 512 * 8; /* (# of PDPT entries) * (# of entries in PD) *
                             (size of entry) */

        /* The initial stack for BSP. We need reserve a large space since Rust
           tend to consume too much memory especially in the debug buid :/  */
        . += 0x10000;
        __kernel_stack = .; /* physical address */

        . = ALIGN(4096);
        __kernel_image_end = .; /* physical address */
        __kernel_heap = .;
        . += 0x1000 * 0x100; /* heap size */
        __kernel_heap_end = .;
    }

    /DISCARD/ : {
        *(*.eh_frame);
        *(*.eh_frame_hdr);
    }
}