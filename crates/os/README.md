Osmium, an archive format
=========================

Note: This document is outdated.

This document will describe Osmium an *archive* format.
Note that Osmium intends to be an archive format and
not an compression, encryption, or anything else format.

These features are left to other formats (like Tarballs
often use gzip for encryption, namely .tar.gz).

First a short note on why Osmium is needed:

There are many archive formats. The most obvious ones to
choose would be Tar or 7z.

Unfortunately, Tar is old and outdated. Especially since
the file sizes and numbers are growing in modern software.
Since we would like to use these archive format for
distributing, for example, software, we need something
scalable, which can quickly handle many files. This is not
something Tar provides. It lacks of an index of files
together with a number of other features.

7z is a more modern archiver in this aspect. However,
7z lacks of a number of important features. For example,
7z does not store file permissions.

This is the reason that I think a new format is needed (
xkcd #972 applies here).


Here we will outline the format of Osmium, an modern,
minimal, and fast file archive.

## The global header

In the start of the Osmium archive, there is a 256 bytes
header. The first 8 bytes describe the version. The version
described in this document is written as "Os 1.0  ". The
next 8 bytes are the size (number of buckets) of the root
node table, which is described in the next section. This
number is like all other numbers in Osmium written in
base-256.

The rest of these are essentially empty bits reserved
for future usage.

## The root table

The root table is an hash map of the root nodes, i.e. the
files and folders in the archive root.

The bucket number of this table is given in the global
header (as described above).

The entry values are filled with relative "pointers" (see
section below) to the file the key describe (for the description
of files and their headers, see the section below).

The table's keys are the node names. This makes it very fast to
lookup a file/directory of a given name.

The table makes use of linear probing with jumps of one.

The excat way this table is stored is by an array in which each
item consists of 8 bytes describing the relative pointer. To find
out if this is the match one has to lseek the pointer and check
against the file name. The excat description of these "pointers"
can be found below.

The key (name) is hashed via the DJB2 hashing algorithm modulo
the bucket number.

### The node "pointers"

These pointers consisting of 8 bytes, describes a position in the
archive file where this given file or chunk of data can be found,
together with a length, and the type of the data it's pointing to.

The pointer consists of:

1) 1 byte of the data type.
2) 5 bytes of position (given per 64 bytes).
3) 2 bytes of length (given per 32 bytes).

Note that the position is relative to the position of the
pointer itself (i.e. the absolute position is given by
the position where the pointer is stored plus the
position that the pointer is carrying).

The type can be one of the following:
- 'f' for file. This means the pointer is pointing to an normal
      file and a file header of course (described below).
- 'd' for directory. This means the pointer is pointing to another
      table of nodes descriping an directory.

The excat specification of these types of data (which is pointed
to) is described here:

#### Files

Files consists of an header (128 bytes). This header is first
a 32 byte file name (not file path!). Then 2 bytes for permission.
2 bytes for the owner's id. 8 bytes for modification and creation
time. The rest is left for future versions. If more header space
is needed the header should end with a '#' indicating that more
header data is given (thus extending the header size with 128
bytes).

Files are ended with EOF (^C) and then zero bytes (^@) as padding.

#### Directories

The directories consists of a 128 bytes header and then a table
of nodes. The table is defined in the same way as the root
table. The bucket size is given by the pointer's size. The
header currently only stores the name of the directory (32
bytes). The rest is reserved for the future.


## On compression, error correction, and encryption

This is not something Osmium will provide. Osmium is intented
to be used in conjunction with other libraries providing
these features.

## Note on incompleteness

This is far from being complete. This specification is only
a vague description of this format.



Sorry for reinventing the wheel again.




- Ticki

