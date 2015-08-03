@echo off
setlocal enabledelayedexpansion

rustc -C relocation-model=dynamic-no-pic -C no-stack-check -O -Z no-landing-pads -A dead-code -W trivial-casts -W trivial-numeric-casts --crate-type lib -o kernel.o --emit obj src/kernel.rs
windows\i386-elf-ld.exe -m elf_i386 -o kernel.bin -T src/kernel.ld kernel.o

rustc -C relocation-model=dynamic-no-pic -C no-stack-check -O -Z no-landing-pads -A dead-code -W trivial-casts -W trivial-numeric-casts --crate-type lib -o example.o --emit obj filesystem/example.rs
windows\i386-elf-ld.exe -m elf_i386 -o filesystem/example.bin -T src/program.ld example.o

echo.> filesystem.asm
set /A i=0
for /f %%f in ('dir /b filesystem') do (
	echo file !i!,"%%f" >> filesystem.asm
	set /A i+=1
)
move filesystem.asm filesystem/filesystem.asm

windows\nasm.exe -f bin -o harddrive.bin -ifilesystem/ -isrc/ src/loader.asm

windows\qemu\qemu-system-i386w.exe -L windows\qemu\Bios -net nic,model=rtl8139 -net user -hda harddrive.bin

pause
