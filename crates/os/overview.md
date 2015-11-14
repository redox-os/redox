Osmium overview:

Global header

- 256 bytes
  - 8 bytes : name and version
  - 8 bytes : base 256 root table bucket number (per 16)
  - 8 bytes : base 256 size of the segment of directory tables (per 16)

Root table
- Size given in global header
  - Consists of entries with:
    - 1 byte : Type identifier
      * 'd' :
        - 8 bytes : position where the table starts in the table segment
        - 7 bytes : Length of the directory table (bucket number)
      * 'f' :
        - 8 bytes : position where the file starts in the file segment
        - 7 bytes : length of the file

The directory tables act in the same way as the root table, only one thing differs, namely that the directory tables got an header of 128 bytes

- 64 bytes of name
- (TODO: Add more info)
- 1 byte which can be either:
  - '#' : indicating that the header is extended ie there are 128 more bytes.
  - '\0' : End of header

The file header (256 bytes)
- 64 bytes of name
- (TODO: Add more info)
- 1 byte which can be either:
  - '#' : indicating that the header is extended ie there are 256 more bytes.
  - '\0' : End of header

After the headers the actual data starts
