cp ../../gregbook/rpng2-x.c ../../gregbook/readpng2.[ch] .
cp ../../gregbook/COPYING ../../gregbook/LICENSE .
cp ../../../*.h .
cp ../../../*.c .
rm pnggccrd.c pngvcrd.c
rm example.c pngtest.c pngw*.c
# change the following 2 lines if zlib is somewhere else
cp ../../../../zlib/*.h .
cp ../../../../zlib/*.c .
rm minigzip.c example.c compress.c deflate.c gz*
