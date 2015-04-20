unfs_root_sector_list:
.parent:
    dq 0
.fragment_number:
    dq 0
.last_fragment:
    dq 0
.next_fragment:
    dq 0
.extents:
    dq (unfs_root_node_list - boot)/512
    dq (unfs_root_node_list.end - unfs_root_node_list)/512

    align 512, db 0
.end:

unfs_root_node_list:
%macro file_node 2+
    %1_node:
    .parent_collection:
        dq (unfs_root_sector_list - boot)/512
    .data_sector_list:
        dq (%1_sector_list - boot)/512
    .data_size:
        dq (%1.end - %1)
    .user_id:
        dq 0
    .group_id:
        dq 0
    .mode:
        dq 0
    .create_time:
        dq 0
    .modify_time:
        dq 0
    .access_time:
        dq 0
    .name:
        db %2,0

        align 512, db 0
    .end:
%endmacro

file_node font_file,"font.unicode.bin"
file_node test_file,"Test"
file_node test_file_two,"Test 2"
file_node test_file_three,"Test 3"

unfs_root_node_list.end:

%macro file_sector_list 1
%1_sector_list:
.parent_node:
    dq (%1_node - boot)/512
.fragment_number:
    dq 0
.last_fragment:
    dq 0
.next_fragment:
    dq 0
.extents:
    dq (%1 - boot)/512
    dq (%1.end - %1)/512

    align 512, db 0
.end:
%endmacro

file_sector_list font_file
file_sector_list test_file
file_sector_list test_file_two
file_sector_list test_file_three

font_file:
incbin "asm/font.unicode.bin"
align 512, db 0
.end:

test_file:
db "This is a test file",0
align 512, db 0
.end:

test_file_two:
db "This is a second test file",0
align 512, db 0
.end:

test_file_three:
incbin "LICENSE.md"
align 512, db 0
.end:
