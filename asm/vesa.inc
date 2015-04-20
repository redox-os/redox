ABSOLUTE 0x5000
VBECardInfo:
	.signature resb 4
	.version resw 1
	.oemstring resd 1
	.capabilities resd 1
	.videomodeptr resd 1
	.totalmemory resw 1
	.oemsoftwarerev resw 1
	.oemvendornameptr resd 1
	.oemproductnameptr resd 1
	.oemproductrevptr resd 1
	.reserved resb 222
	.oemdata resb 256

ABSOLUTE 0x5200
VBEModeInfo:
	.attributes resw 1
	.winA resb 1
	.winB resb 1
	.granularity resw 1
	.winsize resw 1
	.segmentA resw 1
	.segmentB resw 1
	.winfuncptr resd 1
	.bytesperscanline resw 1
	.xresolution resw 1
	.yresolution resw 1
	.xcharsize resb 1
	.ycharsize resb 1
	.numberofplanes resb 1
	.bitsperpixel resb 1
	.numberofbanks resb 1
	.memorymodel resb 1
	.banksize resb 1
	.numberofimagepages resb 1
	.unused resb 1
	.redmasksize resb 1
	.redfieldposition resb 1
	.greenmasksize resb 1
	.greenfieldposition resb 1
	.bluemasksize resb 1
	.bluefieldposition resb 1
	.rsvdmasksize resb 1
	.rsvdfieldposition resb 1
	.directcolormodeinfo resb 1
	.physbaseptr resd 1
	.offscreenmemoryoffset resd 1
	.offscreenmemsize resw 1
	.reserved resb 206

VBE.ModeAttributes:
	.available equ 1 << 0
	.bios equ 1 << 2
	.color equ 1 << 3
	.graphics equ 1 << 4
	.vgacompatible equ 1 << 5
	.notbankable equ 1 << 6
	.linearframebuffer equ 1 << 7
	
ABSOLUTE 0x5400
VBEEDID:
	.header resb 8
	.manufacturer resw 1
	.productid resw 1
	.serial	resd 1
	.manufactureweek	resb 1
	.manufactureyear	resb 1
	.version	resb 1
	.revision	resb 1
	.input	resb 1
	.horizontalsize	resb 1
	.verticalsize	resb 1
	.gamma	resb 1
	.displaytype	resb 1
	.chromaticity	resb 10
	.timingI	resb 1
	.timingII	resb 1
	.timingreserved	resb 1
	.standardtiming:	resw 8	;format: db (horizontal-248)/8, aspectratio | verticalfrequency - 60
		.aspect.16.10	equ 0 	;mul horizontal by 10, shr 4 to get vertical resolution
		.aspect.4.3	equ 1 << 6	;mul horizontal by 3, shr 2 to get vertical resolution
		.aspect.5.4	equ 2 << 6	;shl horizontal by 2, div by 5 to get vertical resolution
		.aspect.16.9	equ 3 << 6	;mul horizontal by 9, shr by 4 to get vertical resolution
	.descriptorblock1	resb 18
	.descriptorblock2	resb 18
	.descriptorblock3	resb 18
	.descriptorblock4	resb 18
	.extensionflag	resb 1
	.checksum	resb 1