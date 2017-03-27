BUILDDIR = $(abspath $(dir $(firstword $(MAKEFILE_LIST))))/build
virtualbox: build/harddrive.bin
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
	$(VBM) modifyvm Redox --memory 1024
	$(VBM) modifyvm Redox --vram 16
	if [ "$(net)" != "no" ]; \
	then \
		$(VBM) modifyvm Redox --nic1 nat; \
		$(VBM) modifyvm Redox --nictype1 82540EM; \
		$(VBM) modifyvm Redox --cableconnected1 on; \
		$(VBM) modifyvm Redox --nictrace1 on; \
		$(VBM) modifyvm Redox --nictracefile1 "$(BUILDDIR)/redox_network.pcap"; \
	fi
	$(VBM) modifyvm Redox --uart1 0x3F8 4
	$(VBM) modifyvm Redox --uartmode1 file "$(BUILDDIR)/redox_serial.log"
	$(VBM) modifyvm Redox --usb off # on
	$(VBM) modifyvm Redox --keyboard ps2
	$(VBM) modifyvm Redox --mouse ps2
	$(VBM) modifyvm Redox --audio $(VB_AUDIO)
	$(VBM) modifyvm Redox --audiocontroller ac97
	$(VBM) modifyvm Redox --nestedpaging on
	echo "Create Disk"
	$(VBM) convertfromraw $< build/harddrive.vdi
	echo "Attach Disk"
	$(VBM) storagectl Redox --name ATA --add sata --controller IntelAHCI --bootable on --portcount 1
	$(VBM) storageattach Redox --storagectl ATA --port 0 --device 0 --type hdd --medium build/harddrive.vdi
	echo "Run VM"
	$(VBM) startvm Redox
