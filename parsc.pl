sub begins_with {
    return substr($_[0], 0, length($_[1])) eq $_[1];
}
@newargs = @ARGV;
$have_o = 0;
# Check if we have -o
for ($i=0;$i<$#ARGV+1;$i++) {
    if ($ARGV[$i] eq "-o") {
        $have_o = 1;
        break;
    }
}
# if we have -o, remove extra-filename
if ($have_o) {
    for ($i=0;$i<$#ARGV+1;$i++) {
        if (begins_with($ARGV[$i], "extra-filename")) {
            undef $newargs[$i];
            undef $newargs[$i-1];
        }
    }
}
for ($i=0;$i<$#newargs+1;$i++) {
    print $newargs[$i] . " " if defined $newargs[$i];
}