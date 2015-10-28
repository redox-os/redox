#!/usr/bin/perl

# Cleans up C++ style comments and converts them to C

while (<>) {
 $line = $_;
 $line =~ s/\/\/([^;]*)(\n)$/\/\*$1 \*\/\n/;
 print $line;
}
