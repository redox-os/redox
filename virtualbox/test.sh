pushd ..
make
popd

echo

echo "Kill VM"
VBoxManage controlvm Redox poweroff
sleep 5

echo

echo "Delete VM"
VBoxManage unregistervm Redox --delete

echo

echo "Create VM"
VBoxManage createvm --name Redox --register
VBoxManage modifyvm Redox --memory 512
VBoxManage modifyvm Redox --vram 64

echo

echo "Create Disk"
VBoxManage convertfromraw ../harddrive.bin Redox.vdi

echo

echo "Attach Disk"
VBoxManage storagectl Redox --name IDE --add ide --controller PIIX4 --bootable on
VBoxManage storageattach Redox --storagectl IDE --port 0 --device 0 --type hdd --medium Redox.vdi

echo

virtualbox --startvm Redox
