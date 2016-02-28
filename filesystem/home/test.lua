#! lua
local file = assert(io.open("test.txt"));
print(file:read("*all"))
file:close()
