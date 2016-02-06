%include "asm/bootsector.asm"

fs_header:
.signature:
    db "REDOXFS",0
.version:
    dq 1
.free_space:
    dq (fs_free_space - boot)/512
    dq (fs_free_space.end - fs_free_space)
.padding:
    align 256, db 0
.extents:
    dq (fs_root_node_list - boot)/512
    dq (fs_root_node_list.end - fs_root_node_list)

    align 512, db 0
.end:

%ifdef ARCH_i386
    %include "asm/startup-i386.asm"
%endif

%ifdef ARCH_x86_64
    %include "asm/startup-x86_64.asm"
%endif
align 512, db 0
startup_end:

;times (0xC000-0x1000)-0x7C00-($-$$) db 0

kernel_file:
  incbin "kernel.bin"
  align 512, db 0
.end:
.length equ kernel_file.end - kernel_file
.length_sectors equ .length / 512

fs_root_node_list:
%macro file 2+
    fs_node.%1:
    .name:
        db %2,0

        align 256, db 0

    .extents:
        dq (fs_data.%1 - boot)/512
        dq (fs_data.%1.end - fs_data.%1)

        align 512, db 0
    .end:
%endmacro

%include "filesystem.gen"

%unmacro file 2+
fs_root_node_list.end:

%macro file 2+
fs_data.%1:
    incbin %2
.end:
    align 512, db 0
%endmacro

%include "filesystem.gen"

%unmacro file 2+

    align 512, db 0
fs_free_space:
    times 16 * 1024 * 1024 db 0  ;16 MB of free space
.end:
