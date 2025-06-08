OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;

SECTIONS
{
    /* 内核整体起始地址 */
    . = BASE_ADDRESS;
    skernel = .;

    /* 文本段 (.text) */
    stext = .;
    .text : {
        *(.text.entry)
        . = ALIGN(4K);
        strampoline = .;
        *(.text.trampoline)
        . = ALIGN(4K);
        *(.text .text.*)
    }

    . = ALIGN(4K);
    etext = .;

    /* 只读数据段 (.rodata) */
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    . = ALIGN(4K);
    erodata = .;

    /* 初始化数据段 (.data) */
    sdata = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    . = ALIGN(4K);
    edata = .;

    /* BSS 段 (.bss) */
    .bss : {
        *(.bss.stack)          /* 可能的手动栈段 */
        sbss = .;              /* 标记 BSS 开始 */
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }
    . = ALIGN(4K);
    ebss = .;                  /* 标记 BSS 结束 */

    /*———— 为内核栈分配空间并导出符号 ————*/
    PROVIDE(sbss_with_stack = .);           /* 栈区起点 */
    . = . + (4096 * 32);                    /* 4096 字节 × 32 个内核栈 */
    PROVIDE(ebss_with_stack = .);           /* 栈区终点 */

    /* 内核结束符号 */
    ekernel = .;

    /* 丢弃调试段 */
    /DISCARD/ : {
        *(.eh_frame)
    }
}