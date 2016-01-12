#!/bin/bash
brew tap homebrew/versions
brew install gcc49
brew tap altkatz/homebrew-gcc_cross_compilers
brew install i386-elf-binutils i386-elf-gcc nasm
