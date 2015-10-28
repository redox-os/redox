#!/usr/bin/perl
#
# Program to take a set of header files and generate DLL export definitions

# Special exports to ignore for this platform

while ( ($file = shift(@ARGV)) ) {
	if ( ! defined(open(FILE, $file)) ) {
		warn "Couldn't open $file: $!\n";
		next;
	}
	$printed_header = 0;
	$file =~ s,.*/,,;
	while (<FILE>) {
		if ( / DECLSPEC.* SDLCALL ([^\s\(]+)/ ) {
			if ( not $exclude{$1} ) {
				print "\t$1\r";
			}
		}
	}
	close(FILE);
}

# Special exports to include for this platform
print "\tSDL_putenv\r";
print "\tSDL_getenv\r";
print "\tSDL_qsort\r";
print "\tSDL_revcpy\r";
print "\tSDL_strlcpy\r";
print "\tSDL_strlcat\r";
print "\tSDL_strdup\r";
print "\tSDL_strrev\r";
print "\tSDL_strupr\r";
print "\tSDL_strlwr\r";
print "\tSDL_ltoa\r";
print "\tSDL_ultoa\r";
print "\tSDL_strcasecmp\r";
print "\tSDL_strncasecmp\r";
print "\tSDL_snprintf\r";
print "\tSDL_vsnprintf\r";
print "\tSDL_iconv\r";
print "\tSDL_iconv_string\r";
print "\tSDL_InitQuickDraw\r";
