for recipe in `ls -1 recipes | grep -v libstd`
do
    ./cook.sh $recipe $*
done
