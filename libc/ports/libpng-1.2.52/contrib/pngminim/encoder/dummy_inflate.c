#include "zlib.h"

int ZEXPORT inflate(strm, flush)
z_streamp strm;
int flush;
{ return Z_OK ; }

int ZEXPORT inflateReset(strm)
z_streamp strm;
{ return Z_OK ; }

int ZEXPORT inflateEnd(strm)
z_streamp strm;
{ return Z_STREAM_ERROR ; }

int ZEXPORT inflateInit_(strm, version, stream_size)
z_streamp strm;
const char *version;
int stream_size;
{ return Z_OK ; }

int ZEXPORT inflateInit2_(strm, windowBits, version, stream_size)
z_streamp strm;
int windowBits;
const char *version;
int stream_size;
{ return Z_STREAM_ERROR ; }
