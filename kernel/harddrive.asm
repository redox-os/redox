%include "asm/bootsector.asm"

startup_start:
%ifdef ARCH_i386
    %include "asm/startup-i386.asm"
%endif

%ifdef ARCH_x86_64
    %include "asm/startup-x86_64.asm"
%endif
align 512, db 0
startup_end:

kernel_file:
  incbin "kernel.bin"
  align 512, db 0
.end:
.length equ kernel_file.end - kernel_file
.length_sectors equ .length / 512

real_fs:
incbin "filesystem.bin"
real_fs.end:
