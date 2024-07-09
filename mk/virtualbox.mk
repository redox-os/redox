# Configuration file for VirtualBox, it creates a VirtualBox virtual machine

virtualbox: $(BUILD)/harddrive.img
	echo "Delete VM"
	-$(VBM) unregistervm Redox --delete; \
	if [ $$? -ne 0 ]; \
	then \
		if [ -d "$$HOME/VirtualBox VMs/Redox" ]; \
		then \
			echo "Redox directory exists, deleting..."; \
			$(RM) -rf "$$HOME/VirtualBox VMs/Redox"; \
		fi \
	fi
	echo "Delete Disk"
	-$(RM) harddrive.vdi
	echo "Create VM"
	$(VBM) createvm --name Redox --register
	echo "Set Configuration"
	$(VBM) modifyvm Redox --memory 2048
	$(VBM) modifyvm Redox --vram 32
	if [ "$(net)" != "no" ]; \
	then \
		$(VBM) modifyvm Redox --nic1 nat; \
		$(VBM) modifyvm Redox --nictype1 82540EM; \
		$(VBM) modifyvm Redox --cableconnected1 on; \
		$(VBM) modifyvm Redox --nictrace1 on; \
		$(VBM) modifyvm Redox --nictracefile1 "$(ROOT)/$(BUILD)/network.pcap"; \
	fi
	$(VBM) modifyvm Redox --uart1 0x3F8 4
	$(VBM) modifyvm Redox --uartmode1 file "$(ROOT)/$(BUILD)/serial.log"
	$(VBM) modifyvm Redox --usb off # on
	$(VBM) modifyvm Redox --keyboard ps2
	$(VBM) modifyvm Redox --mouse ps2
	$(VBM) modifyvm Redox --audio $(VB_AUDIO)
	$(VBM) modifyvm Redox --audiocontroller hda
	$(VBM) modifyvm Redox --audioout on
	$(VBM) modifyvm Redox --nestedpaging on
	echo "Create Disk"
	$(VBM) convertfromraw $< $(BUILD)/harddrive.vdi
	echo "Attach Disk"
	$(VBM) storagectl Redox --name ATA --add sata --controller IntelAHCI --bootable on --portcount 1
	$(VBM) storageattach Redox --storagectl ATA --port 0 --device 0 --type hdd --medium $(BUILD)/harddrive.vdi
	echo "Run VM"
	$(VBM) startvm Redox
