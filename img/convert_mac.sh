if [ -d "$1" ]
then
  for file in `find $1 -type f`
  do
    mkdir -p $(dirname $file | sed "s|$1|$1_bmp|")
    sips -s format bmp -d profile $file --out $(echo $file | sed "s|$1|$1_bmp|" | sed 's|png|bmp|')
  done
else
  echo $0 [Directory]
fi
