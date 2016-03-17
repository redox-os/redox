#! lua
for i=0,255,1
do
  print(string.char(27)..'[38;5;'..i..'m'..i..string.char(27)..'[0m')
end
