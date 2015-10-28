;
;    SDL - Simple DirectMedia Layer
;    Copyright (C) 1997-2012 Sam Lantinga
;
;    This library is free software; you can redistribute it and/or
;    modify it under the terms of the GNU Library General Public
;    License as published by the Free Software Foundation; either
;    version 2 of the License, or (at your option) any later version.
;
;    This library is distributed in the hope that it will be useful,
;    but WITHOUT ANY WARRANTY; without even the implied warranty of
;    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
;    Library General Public License for more details.
;
;    You should have received a copy of the GNU Library General Public
;    License along with this library; if not, write to the Free
;    Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA
;
;    Sam Lantinga
;    slouken@libsdl.org
;
; Assembler routines for RISC OS display
;

	AREA |C$$CODE|

	EXPORT |RISCOS_Put32|

; Display 32bpp to 32bpp, 1:1
;
; Code provided by Adrain Lees
;
; entry a1 -> destination
;       a2 =  dest width in pixels
;       a3 =  dest line length in bytes
;       a4 =  dest height in scanlines
;       arg5 -> source
;       arg6 =  byte offset from end of source line to start of next

Arg5    *       10*4
Arg6    *       Arg5+4

RISCOS_Put32    ROUT
                STMFD   sp!,{a2,v1-v6,sl,fp,lr}
                LDR     ip,[sp,#Arg5]
                MOV     lr,a1
                B       ucp64lp

00              ;tail strip of 1-15 pixels

                LDR     v1,[ip],#4
01              SUBS    a2,a2,#1
                STR     v1,[lr],#4
                LDRHI   v1,[ip],#4
                BHI     %01
                B       %02

ucp64end        ADDS    a2,a2,#16
                BNE     %00

02              SUBS    a4,a4,#1                ;height--
                LDRHI   v1,[sp,#Arg6]
                LDRHI   a2,[sp]                 ;reload width
                BLS     %03

                ;move to start of next scanline

                ADD     lr,a1,a3
                ADD     a1,a1,a3
                ADD     ip,ip,v1

ucp64lp         SUBS    a2,a2,#16
                BLO     ucp64end

                PLD     [ip,#64]

                LDR     v1,[ip],#4
                LDR     v2,[ip],#4
                LDR     v3,[ip],#4
                LDR     v4,[ip],#4
                LDR     v5,[ip],#4
                LDR     v6,[ip],#4
                LDR     sl,[ip],#4
                LDR     fp,[ip],#4
                STR     v1,[lr],#4
                STR     v2,[lr],#4
                STR     v3,[lr],#4
                STR     v4,[lr],#4
                STR     v5,[lr],#4
                STR     v6,[lr],#4
                STR     sl,[lr],#4
                STR     fp,[lr],#4

                PLD     [ip,#64]

                LDR     v1,[ip],#4
                LDR     v2,[ip],#4
                LDR     v3,[ip],#4
                LDR     v4,[ip],#4
                LDR     v5,[ip],#4
                LDR     v6,[ip],#4
                LDR     sl,[ip],#4
                LDR     fp,[ip],#4
                STR     v1,[lr],#4
                STR     v2,[lr],#4
                STR     v3,[lr],#4
                STR     v4,[lr],#4
                STR     v5,[lr],#4
                STR     v6,[lr],#4
                STR     sl,[lr],#4
                STR     fp,[lr],#4

                B       ucp64lp

03              LDMFD   sp!,{a2,v1-v6,sl,fp,pc}

