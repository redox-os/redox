/*
 *  Copyright (C) 2002-2010  The DOSBox Team
 *
 *  This program is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation; either version 2 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program; if not, write to the Free Software
 *  Foundation, Inc., 59 Temple Place - Suite 330, Boston, MA 02111-1307, USA.
 */

/* $Id: cpu.cpp,v 1.116 2009-03-16 18:10:08 c2woody Exp $ */

#include <assert.h>
#include <sstream>
#include "dosbox.h"
#include "cpu.h"
//#include "memory.h"
#include "debug.h"
#include "mapper.h"
#include "setup.h"
#include "programs.h"
#include "paging.h"
#include "lazyflags.h"
#include "support.h"

Bitu DEBUG_EnableDebugger(void);
extern void GFX_SetTitle(Bit32s cycles ,Bits frameskip,bool paused);

#if 1
#undef LOG
#if defined (_MSC_VER)
#define LOG(X,Y)
#else
#define LOG(X,Y) CPU_LOG
#define CPU_LOG(...)
#endif
#endif

CPU_Regs cpu_regs;
CPUBlock cpu;
Segments Segs;

Bit32s CPU_Cycles = 0;
Bit32s CPU_CycleLeft = 3000;
Bit32s CPU_CycleMax = 3000;
Bit32s CPU_OldCycleMax = 3000;
Bit32s CPU_CyclePercUsed = 100;
Bit32s CPU_CycleLimit = -1;
Bit32s CPU_CycleUp = 0;
Bit32s CPU_CycleDown = 0;
Bit64s CPU_IODelayRemoved = 0;
CPU_Decoder * cpudecoder;
bool CPU_CycleAutoAdjust = false;
bool CPU_SkipCycleAutoAdjust = false;
Bitu CPU_AutoDetermineMode = 0;

Bitu CPU_ArchitectureType = CPU_ARCHTYPE_MIXED;

Bitu CPU_flag_id_toggle=0;

Bitu CPU_PrefetchQueueSize=0;

void CPU_Core_Full_Init(void);
void CPU_Core_Normal_Init(void);
void CPU_Core_Simple_Init(void);
#if (C_DYNAMIC_X86)
void CPU_Core_Dyn_X86_Init(void);
void CPU_Core_Dyn_X86_Cache_Init(bool enable_cache);
void CPU_Core_Dyn_X86_Cache_Close(void);
void CPU_Core_Dyn_X86_SetFPUMode(bool dh_fpu);
#elif (C_DYNREC)
void CPU_Core_Dynrec_Init(void);
void CPU_Core_Dynrec_Cache_Init(bool enable_cache);
void CPU_Core_Dynrec_Cache_Close(void);
#endif

/* In debug mode exceptions are tested and dosbox exits when
 * a unhandled exception state is detected.
 * USE CHECK_EXCEPT to raise an exception in that case to see if that exception
 * solves the problem.
 *
 * In non-debug mode dosbox doesn't do detection (and hence doesn't crash at
 * that point). (game might crash later due to the unhandled exception) */

#if C_DEBUG
// #define CPU_CHECK_EXCEPT 1
// #define CPU_CHECK_IGNORE 1
 /* Use CHECK_EXCEPT when something doesn't work to see if a exception is
 * needed that isn't enabled by default.*/
#else
/* NORMAL NO CHECKING => More Speed */
#define CPU_CHECK_IGNORE 1
#endif /* C_DEBUG */

#if defined(CPU_CHECK_IGNORE)
#define CPU_CHECK_COND(cond,msg,exc,sel) {	\
	if (cond) do {} while (0);				\
}
#elif defined(CPU_CHECK_EXCEPT)
#define CPU_CHECK_COND(cond,msg,exc,sel) {	\
	if (cond) {					\
		CPU_Exception(exc,sel);		\
		return;				\
	}					\
}
#else
#define CPU_CHECK_COND(cond,msg,exc,sel) {	\
	if (cond) E_Exit(msg);			\
}
#endif


void Descriptor::Load(PhysPt address) {
	cpu.mpl=0;
	Bit32u* data = (Bit32u*)&saved;
	*data	  = mem_readd(address);
	*(data+1) = mem_readd(address+4);
	cpu.mpl=3;
}
void Descriptor:: Save(PhysPt address) {
	cpu.mpl=0;
	Bit32u* data = (Bit32u*)&saved;
	mem_writed(address,*data);
	mem_writed(address+4,*(data+1));
	cpu.mpl=03;
}


void CPU_Push16(Bitu value) {
	Bit32u new_esp=(reg_esp&cpu.stack.notmask)|((reg_esp-2)&cpu.stack.mask);
	mem_writew(SegPhys(ss) + (new_esp & cpu.stack.mask) ,value);
	reg_esp=new_esp;
}

void CPU_Push32(Bitu value) {
	Bit32u new_esp=(reg_esp&cpu.stack.notmask)|((reg_esp-4)&cpu.stack.mask);
	mem_writed(SegPhys(ss) + (new_esp & cpu.stack.mask) ,value);
	reg_esp=new_esp;
}

Bitu CPU_Pop16(void) {
	Bitu val=mem_readw(SegPhys(ss) + (reg_esp & cpu.stack.mask));
	reg_esp=(reg_esp&cpu.stack.notmask)|((reg_esp+2)&cpu.stack.mask);
	return val;
}

Bitu CPU_Pop32(void) {
	Bitu val=mem_readd(SegPhys(ss) + (reg_esp & cpu.stack.mask));
	reg_esp=(reg_esp&cpu.stack.notmask)|((reg_esp+4)&cpu.stack.mask);
	return val;
}

PhysPt SelBase(Bitu sel) {
	if (cpu.cr0 & CR0_PROTECTION) {
		Descriptor desc;
		cpu.gdt.GetDescriptor(sel,desc);
		return desc.GetBase();
	} else {
		return sel<<4;
	}
}


void CPU_SetFlags(Bitu word,Bitu mask) {
	mask|=CPU_flag_id_toggle;	// ID-flag can be toggled on cpuid-supporting CPUs
	reg_flags=(reg_flags & ~mask)|(word & mask)|2;
	cpu.direction=1-((reg_flags & FLAG_DF) >> 9);
}

bool CPU_PrepareException(Bitu which,Bitu error) {
	cpu.exception.which=which;
	cpu.exception.error=error;
	return true;
}

bool CPU_CLI(void) {
	if (cpu.pmode && ((!GETFLAG(VM) && (GETFLAG_IOPL<cpu.cpl)) || (GETFLAG(VM) && (GETFLAG_IOPL<3)))) {
		return CPU_PrepareException(EXCEPTION_GP,0);
	} else {
		SETFLAGBIT(IF,false);
		return false;
	}
}

bool CPU_STI(void) {
	if (cpu.pmode && ((!GETFLAG(VM) && (GETFLAG_IOPL<cpu.cpl)) || (GETFLAG(VM) && (GETFLAG_IOPL<3)))) {
		return CPU_PrepareException(EXCEPTION_GP,0);
	} else {
 		SETFLAGBIT(IF,true);
		return false;
	}
}

bool CPU_POPF(Bitu use32) {
	if (cpu.pmode && GETFLAG(VM) && (GETFLAG(IOPL)!=FLAG_IOPL)) {
		/* Not enough privileges to execute POPF */
		return CPU_PrepareException(EXCEPTION_GP,0);
	}
	Bitu mask=FMASK_ALL;
	/* IOPL field can only be modified when CPL=0 or in real mode: */
	if (cpu.pmode && (cpu.cpl>0)) mask &= (~FLAG_IOPL);
	if (cpu.pmode && !GETFLAG(VM) && (GETFLAG_IOPL<cpu.cpl)) mask &= (~FLAG_IF);
	if (use32)
		CPU_SetFlags(CPU_Pop32(),mask);
	else CPU_SetFlags(CPU_Pop16(),mask & 0xffff);
	DestroyConditionFlags();
	return false;
}

bool CPU_PUSHF(Bitu use32) {
	if (cpu.pmode && GETFLAG(VM) && (GETFLAG(IOPL)!=FLAG_IOPL)) {
		/* Not enough privileges to execute PUSHF */
		return CPU_PrepareException(EXCEPTION_GP,0);
	}
	FillFlags();
	if (use32)
		CPU_Push32(reg_flags & 0xfcffff);
	else CPU_Push16(reg_flags);
	return false;
}

void CPU_CheckSegments(void) {
	bool needs_invalidation=false;
	Descriptor desc;
	if (!cpu.gdt.GetDescriptor(SegValue(es),desc)) needs_invalidation=true;
	else switch (desc.Type()) {
		case DESC_DATA_EU_RO_NA:	case DESC_DATA_EU_RO_A:	case DESC_DATA_EU_RW_NA:	case DESC_DATA_EU_RW_A:
		case DESC_DATA_ED_RO_NA:	case DESC_DATA_ED_RO_A:	case DESC_DATA_ED_RW_NA:	case DESC_DATA_ED_RW_A:
		case DESC_CODE_N_NC_A:	case DESC_CODE_N_NC_NA:	case DESC_CODE_R_NC_A:	case DESC_CODE_R_NC_NA:
			if (cpu.cpl>desc.DPL()) needs_invalidation=true; break;
		default: break;	}
	if (needs_invalidation) CPU_SetSegGeneral(es,0);

	needs_invalidation=false;
	if (!cpu.gdt.GetDescriptor(SegValue(ds),desc)) needs_invalidation=true;
	else switch (desc.Type()) {
		case DESC_DATA_EU_RO_NA:	case DESC_DATA_EU_RO_A:	case DESC_DATA_EU_RW_NA:	case DESC_DATA_EU_RW_A:
		case DESC_DATA_ED_RO_NA:	case DESC_DATA_ED_RO_A:	case DESC_DATA_ED_RW_NA:	case DESC_DATA_ED_RW_A:
		case DESC_CODE_N_NC_A:	case DESC_CODE_N_NC_NA:	case DESC_CODE_R_NC_A:	case DESC_CODE_R_NC_NA:
			if (cpu.cpl>desc.DPL()) needs_invalidation=true; break;
		default: break;	}
	if (needs_invalidation) CPU_SetSegGeneral(ds,0);

	needs_invalidation=false;
	if (!cpu.gdt.GetDescriptor(SegValue(fs),desc)) needs_invalidation=true;
	else switch (desc.Type()) {
		case DESC_DATA_EU_RO_NA:	case DESC_DATA_EU_RO_A:	case DESC_DATA_EU_RW_NA:	case DESC_DATA_EU_RW_A:
		case DESC_DATA_ED_RO_NA:	case DESC_DATA_ED_RO_A:	case DESC_DATA_ED_RW_NA:	case DESC_DATA_ED_RW_A:
		case DESC_CODE_N_NC_A:	case DESC_CODE_N_NC_NA:	case DESC_CODE_R_NC_A:	case DESC_CODE_R_NC_NA:
			if (cpu.cpl>desc.DPL()) needs_invalidation=true; break;
		default: break;	}
	if (needs_invalidation) CPU_SetSegGeneral(fs,0);

	needs_invalidation=false;
	if (!cpu.gdt.GetDescriptor(SegValue(gs),desc)) needs_invalidation=true;
	else switch (desc.Type()) {
		case DESC_DATA_EU_RO_NA:	case DESC_DATA_EU_RO_A:	case DESC_DATA_EU_RW_NA:	case DESC_DATA_EU_RW_A:
		case DESC_DATA_ED_RO_NA:	case DESC_DATA_ED_RO_A:	case DESC_DATA_ED_RW_NA:	case DESC_DATA_ED_RW_A:
		case DESC_CODE_N_NC_A:	case DESC_CODE_N_NC_NA:	case DESC_CODE_R_NC_A:	case DESC_CODE_R_NC_NA:
			if (cpu.cpl>desc.DPL()) needs_invalidation=true; break;
		default: break;	}
	if (needs_invalidation) CPU_SetSegGeneral(gs,0);
}


class TaskStateSegment {
public:
	TaskStateSegment() {
		valid=false;
	}
	bool IsValid(void) {
		return valid;
	}
	Bitu Get_back(void) {
		cpu.mpl=0;
		Bit16u backlink=mem_readw(base);
		cpu.mpl=3;
		return backlink;
	}
	void SaveSelector(void) {
		cpu.gdt.SetDescriptor(selector,desc);
	}
	void Get_SSx_ESPx(Bitu level,Bitu & _ss,Bitu & _esp) {
		cpu.mpl=0;
		if (is386) {
			PhysPt where=base+offsetof(TSS_32,esp0)+level*8;
			_esp=mem_readd(where);
			_ss=mem_readw(where+4);
		} else {
			PhysPt where=base+offsetof(TSS_16,sp0)+level*4;
			_esp=mem_readw(where);
			_ss=mem_readw(where+2);
		}
		cpu.mpl=3;
	}
	bool SetSelector(Bitu new_sel) {
		valid=false;
		if ((new_sel & 0xfffc)==0) {
			selector=0;
			base=0;
			limit=0;
			is386=1;
			return true;
		}
		if (new_sel&4) return false;
		if (!cpu.gdt.GetDescriptor(new_sel,desc)) return false;
		switch (desc.Type()) {
			case DESC_286_TSS_A:		case DESC_286_TSS_B:
			case DESC_386_TSS_A:		case DESC_386_TSS_B:
				break;
			default:
				return false;
		}
		if (!desc.saved.seg.p) return false;
		selector=new_sel;
		valid=true;
		base=desc.GetBase();
		limit=desc.GetLimit();
		is386=desc.Is386();
		return true;
	}
	TSS_Descriptor desc;
	Bitu selector;
	PhysPt base;
	Bitu limit;
	Bitu is386;
	bool valid;
};

TaskStateSegment cpu_tss;

enum TSwitchType {
	TSwitch_JMP,TSwitch_CALL_INT,TSwitch_IRET
};

bool CPU_SwitchTask(Bitu new_tss_selector,TSwitchType tstype,Bitu old_eip) {
	FillFlags();
	TaskStateSegment new_tss;
	if (!new_tss.SetSelector(new_tss_selector))
		E_Exit("Illegal TSS for switch, selector=%x, switchtype=%x",new_tss_selector,tstype);
	if (tstype==TSwitch_IRET) {
		if (!new_tss.desc.IsBusy())
			E_Exit("TSS not busy for IRET");
	} else {
		if (new_tss.desc.IsBusy())
			E_Exit("TSS busy for JMP/CALL/INT");
	}
	Bitu new_cr3=0;
	Bitu new_eax,new_ebx,new_ecx,new_edx,new_esp,new_ebp,new_esi,new_edi;
	Bitu new_es,new_cs,new_ss,new_ds,new_fs,new_gs;
	Bitu new_ldt,new_eip,new_eflags;
	/* Read new context from new TSS */
	if (new_tss.is386) {
		new_cr3=mem_readd(new_tss.base+offsetof(TSS_32,cr3));
		new_eip=mem_readd(new_tss.base+offsetof(TSS_32,eip));
		new_eflags=mem_readd(new_tss.base+offsetof(TSS_32,eflags));
		new_eax=mem_readd(new_tss.base+offsetof(TSS_32,eax));
		new_ecx=mem_readd(new_tss.base+offsetof(TSS_32,ecx));
		new_edx=mem_readd(new_tss.base+offsetof(TSS_32,edx));
		new_ebx=mem_readd(new_tss.base+offsetof(TSS_32,ebx));
		new_esp=mem_readd(new_tss.base+offsetof(TSS_32,esp));
		new_ebp=mem_readd(new_tss.base+offsetof(TSS_32,ebp));
		new_edi=mem_readd(new_tss.base+offsetof(TSS_32,edi));
		new_esi=mem_readd(new_tss.base+offsetof(TSS_32,esi));

		new_es=mem_readw(new_tss.base+offsetof(TSS_32,es));
		new_cs=mem_readw(new_tss.base+offsetof(TSS_32,cs));
		new_ss=mem_readw(new_tss.base+offsetof(TSS_32,ss));
		new_ds=mem_readw(new_tss.base+offsetof(TSS_32,ds));
		new_fs=mem_readw(new_tss.base+offsetof(TSS_32,fs));
		new_gs=mem_readw(new_tss.base+offsetof(TSS_32,gs));
		new_ldt=mem_readw(new_tss.base+offsetof(TSS_32,ldt));
	} else {
		E_Exit("286 task switch");
		new_cr3=0;
		new_eip=0;
		new_eflags=0;
		new_eax=0;	new_ecx=0;	new_edx=0;	new_ebx=0;
		new_esp=0;	new_ebp=0;	new_edi=0;	new_esi=0;

		new_es=0;	new_cs=0;	new_ss=0;	new_ds=0;	new_fs=0;	new_gs=0;
		new_ldt=0;
	}

	/* Check if we need to clear busy bit of old TASK */
	if (tstype==TSwitch_JMP || tstype==TSwitch_IRET) {
		cpu_tss.desc.SetBusy(false);
		cpu_tss.SaveSelector();
	}
	Bit32u old_flags = reg_flags;
	if (tstype==TSwitch_IRET) old_flags &= (~FLAG_NT);

	/* Save current context in current TSS */
	if (cpu_tss.is386) {
		mem_writed(cpu_tss.base+offsetof(TSS_32,eflags),old_flags);
		mem_writed(cpu_tss.base+offsetof(TSS_32,eip),old_eip);

		mem_writed(cpu_tss.base+offsetof(TSS_32,eax),reg_eax);
		mem_writed(cpu_tss.base+offsetof(TSS_32,ecx),reg_ecx);
		mem_writed(cpu_tss.base+offsetof(TSS_32,edx),reg_edx);
		mem_writed(cpu_tss.base+offsetof(TSS_32,ebx),reg_ebx);
		mem_writed(cpu_tss.base+offsetof(TSS_32,esp),reg_esp);
		mem_writed(cpu_tss.base+offsetof(TSS_32,ebp),reg_ebp);
		mem_writed(cpu_tss.base+offsetof(TSS_32,esi),reg_esi);
		mem_writed(cpu_tss.base+offsetof(TSS_32,edi),reg_edi);

		mem_writed(cpu_tss.base+offsetof(TSS_32,es),SegValue(es));
		mem_writed(cpu_tss.base+offsetof(TSS_32,cs),SegValue(cs));
		mem_writed(cpu_tss.base+offsetof(TSS_32,ss),SegValue(ss));
		mem_writed(cpu_tss.base+offsetof(TSS_32,ds),SegValue(ds));
		mem_writed(cpu_tss.base+offsetof(TSS_32,fs),SegValue(fs));
		mem_writed(cpu_tss.base+offsetof(TSS_32,gs),SegValue(gs));
	} else {
		E_Exit("286 task switch");
	}

	/* Setup a back link to the old TSS in new TSS */
	if (tstype==TSwitch_CALL_INT) {
		if (new_tss.is386) {
			mem_writed(new_tss.base+offsetof(TSS_32,back),cpu_tss.selector);
		} else {
			mem_writew(new_tss.base+offsetof(TSS_16,back),cpu_tss.selector);
		}
		/* And make the new task's eflag have the nested task bit */
		new_eflags|=FLAG_NT;
	}
	/* Set the busy bit in the new task */
	if (tstype==TSwitch_JMP || tstype==TSwitch_CALL_INT) {
		new_tss.desc.SetBusy(true);
		new_tss.SaveSelector();
	}

//	cpu.cr0|=CR0_TASKSWITCHED;
	if (new_tss_selector == cpu_tss.selector) {
		reg_eip = old_eip;
		new_cs = SegValue(cs);
		new_ss = SegValue(ss);
		new_ds = SegValue(ds);
		new_es = SegValue(es);
		new_fs = SegValue(fs);
		new_gs = SegValue(gs);
	} else {

		/* Setup the new cr3 */
		PAGING_SetDirBase(new_cr3);

		/* Load new context */
		if (new_tss.is386) {
			reg_eip=new_eip;
			CPU_SetFlags(new_eflags,FMASK_ALL | FLAG_VM);
			reg_eax=new_eax;
			reg_ecx=new_ecx;
			reg_edx=new_edx;
			reg_ebx=new_ebx;
			reg_esp=new_esp;
			reg_ebp=new_ebp;
			reg_edi=new_edi;
			reg_esi=new_esi;

//			new_cs=mem_readw(new_tss.base+offsetof(TSS_32,cs));
		} else {
			E_Exit("286 task switch");
		}
	}
	/* Load the new selectors */
	if (reg_flags & FLAG_VM) {
		SegSet16(cs,new_cs);
		cpu.code.big=false;
		cpu.cpl=3;			//We don't have segment caches so this will do
	} else {
		/* Protected mode task */
		if (new_ldt!=0) CPU_LLDT(new_ldt);
		/* Load the new CS*/
		Descriptor cs_desc;
		cpu.cpl=new_cs & 3;
		if (!cpu.gdt.GetDescriptor(new_cs,cs_desc))
			E_Exit("Task switch with CS beyond limits");
		if (!cs_desc.saved.seg.p)
			E_Exit("Task switch with non present code-segment");
		switch (cs_desc.Type()) {
		case DESC_CODE_N_NC_A:		case DESC_CODE_N_NC_NA:
		case DESC_CODE_R_NC_A:		case DESC_CODE_R_NC_NA:
			if (cpu.cpl != cs_desc.DPL()) E_Exit("Task CS RPL != DPL");
			goto doconforming;
		case DESC_CODE_N_C_A:		case DESC_CODE_N_C_NA:
		case DESC_CODE_R_C_A:		case DESC_CODE_R_C_NA:
			if (cpu.cpl < cs_desc.DPL()) E_Exit("Task CS RPL < DPL");
doconforming:
			Segs.phys[cs]=cs_desc.GetBase();
			cpu.code.big=cs_desc.Big()>0;
			Segs.val[cs]=new_cs;
			break;
		default:
			E_Exit("Task switch CS Type %d",cs_desc.Type());
		}
	}
	CPU_SetSegGeneral(es,new_es);
	CPU_SetSegGeneral(ss,new_ss);
	CPU_SetSegGeneral(ds,new_ds);
	CPU_SetSegGeneral(fs,new_fs);
	CPU_SetSegGeneral(gs,new_gs);
	if (!cpu_tss.SetSelector(new_tss_selector)) {
		LOG(LOG_CPU,LOG_NORMAL)("TaskSwitch: set tss selector %X failed",new_tss_selector);
	}
//	cpu_tss.desc.SetBusy(true);
//	cpu_tss.SaveSelector();
//	LOG_MSG("Task CPL %X CS:%X IP:%X SS:%X SP:%X eflags %x",cpu.cpl,SegValue(cs),reg_eip,SegValue(ss),reg_esp,reg_flags);
	return true;
}

bool CPU_IO_Exception(Bitu port,Bitu size) {
	if (cpu.pmode && ((GETFLAG_IOPL<cpu.cpl) || GETFLAG(VM))) {
		cpu.mpl=0;
		if (!cpu_tss.is386) goto doexception;
		PhysPt bwhere=cpu_tss.base+0x66;
		Bitu ofs=mem_readw(bwhere);
		if (ofs>cpu_tss.limit) goto doexception;
		bwhere=cpu_tss.base+ofs+(port/8);
		Bitu map=mem_readw(bwhere);
		Bitu mask=(0xffff>>(16-size)) << (port&7);
		if (map & mask) goto doexception;
		cpu.mpl=3;
	}
	return false;
doexception:
	cpu.mpl=3;
	LOG(LOG_CPU,LOG_NORMAL)("IO Exception port %X",port);
	return CPU_PrepareException(EXCEPTION_GP,0);
}

void CPU_Exception(Bitu which,Bitu error ) {
//	LOG_MSG("Exception %d error %x",which,error);
	cpu.exception.error=error;
	CPU_Interrupt(which,CPU_INT_EXCEPTION | ((which>=8) ? CPU_INT_HAS_ERROR : 0),reg_eip);
}

Bit8u lastint;
void CPU_Interrupt(Bitu num,Bitu type,Bitu oldeip) {
	lastint=num;
	FillFlags();
#if C_DEBUG
	switch (num) {
	case 0xcd:
#if C_HEAVY_DEBUG
 		LOG(LOG_CPU,LOG_ERROR)("Call to interrupt 0xCD this is BAD");
		DEBUG_HeavyWriteLogInstruction();
		E_Exit("Call to interrupt 0xCD this is BAD");
#endif
		break;
	case 0x03:
		if (DEBUG_Breakpoint()) {
			CPU_Cycles=0;
			return;
		}
	};
#endif
	if (!cpu.pmode) {
		/* Save everything on a 16-bit stack */
		CPU_Push16(reg_flags & 0xffff);
		CPU_Push16(SegValue(cs));
		CPU_Push16(oldeip);
		SETFLAGBIT(IF,false);
		SETFLAGBIT(TF,false);
		/* Get the new CS:IP from vector table */
		PhysPt base=cpu.idt.GetBase();
		reg_eip=mem_readw(base+(num << 2));
		Segs.val[cs]=mem_readw(base+(num << 2)+2);
		Segs.phys[cs]=Segs.val[cs]<<4;
		cpu.code.big=false;
		return;
	} else {
		/* Protected Mode Interrupt */
		if ((reg_flags & FLAG_VM) && (type&CPU_INT_SOFTWARE) && !(type&CPU_INT_NOIOPLCHECK)) {
//			LOG_MSG("Software int in v86, AH %X IOPL %x",reg_ah,(reg_flags & FLAG_IOPL) >>12);
			if ((reg_flags & FLAG_IOPL)!=FLAG_IOPL) {
				CPU_Exception(EXCEPTION_GP,0);
				return;
			}
		}

		Descriptor gate;
		if (!cpu.idt.GetDescriptor(num<<3,gate)) {
			// zone66
			CPU_Exception(EXCEPTION_GP,num*8+2+(type&CPU_INT_SOFTWARE)?0:1);
			return;
		}

		if ((type&CPU_INT_SOFTWARE) && (gate.DPL()<cpu.cpl)) {
			// zone66, win3.x e
			CPU_Exception(EXCEPTION_GP,num*8+2);
			return;
		}


		switch (gate.Type()) {
		case DESC_286_INT_GATE:		case DESC_386_INT_GATE:
		case DESC_286_TRAP_GATE:	case DESC_386_TRAP_GATE:
			{
				CPU_CHECK_COND(!gate.saved.seg.p,
					"INT:Gate segment not present",
					EXCEPTION_NP,num*8+2+(type&CPU_INT_SOFTWARE)?0:1)

				Descriptor cs_desc;
				Bitu gate_sel=gate.GetSelector();
				Bitu gate_off=gate.GetOffset();
				CPU_CHECK_COND((gate_sel & 0xfffc)==0,
					"INT:Gate with CS zero selector",
					EXCEPTION_GP,(type&CPU_INT_SOFTWARE)?0:1)
				CPU_CHECK_COND(!cpu.gdt.GetDescriptor(gate_sel,cs_desc),
					"INT:Gate with CS beyond limit",
					EXCEPTION_GP,(gate_sel & 0xfffc)+(type&CPU_INT_SOFTWARE)?0:1)

				Bitu cs_dpl=cs_desc.DPL();
				CPU_CHECK_COND(cs_dpl>cpu.cpl,
					"Interrupt to higher privilege",
					EXCEPTION_GP,(gate_sel & 0xfffc)+(type&CPU_INT_SOFTWARE)?0:1)
				switch (cs_desc.Type()) {
				case DESC_CODE_N_NC_A:	case DESC_CODE_N_NC_NA:
				case DESC_CODE_R_NC_A:	case DESC_CODE_R_NC_NA:
					if (cs_dpl<cpu.cpl) {
						/* Prepare for gate to inner level */
						CPU_CHECK_COND(!cs_desc.saved.seg.p,
							"INT:Inner level:CS segment not present",
							EXCEPTION_NP,(gate_sel & 0xfffc)+(type&CPU_INT_SOFTWARE)?0:1)
						CPU_CHECK_COND((reg_flags & FLAG_VM) && (cs_dpl!=0),
							"V86 interrupt calling codesegment with DPL>0",
							EXCEPTION_GP,gate_sel & 0xfffc)

						Bitu n_ss,n_esp;
						Bitu o_ss,o_esp;
						o_ss=SegValue(ss);
						o_esp=reg_esp;
						cpu_tss.Get_SSx_ESPx(cs_dpl,n_ss,n_esp);
						CPU_CHECK_COND((n_ss & 0xfffc)==0,
							"INT:Gate with SS zero selector",
							EXCEPTION_TS,(type&CPU_INT_SOFTWARE)?0:1)
						Descriptor n_ss_desc;
						CPU_CHECK_COND(!cpu.gdt.GetDescriptor(n_ss,n_ss_desc),
							"INT:Gate with SS beyond limit",
							EXCEPTION_TS,(n_ss & 0xfffc)+(type&CPU_INT_SOFTWARE)?0:1)
						CPU_CHECK_COND(((n_ss & 3)!=cs_dpl) || (n_ss_desc.DPL()!=cs_dpl),
							"INT:Inner level with CS_DPL!=SS_DPL and SS_RPL",
							EXCEPTION_TS,(n_ss & 0xfffc)+(type&CPU_INT_SOFTWARE)?0:1)

						// check if stack segment is a writable data segment
						switch (n_ss_desc.Type()) {
						case DESC_DATA_EU_RW_NA:		case DESC_DATA_EU_RW_A:
						case DESC_DATA_ED_RW_NA:		case DESC_DATA_ED_RW_A:
							break;
						default:
							E_Exit("INT:Inner level:Stack segment not writable.");		// or #TS(ss_sel+EXT)
						}
						CPU_CHECK_COND(!n_ss_desc.saved.seg.p,
							"INT:Inner level with nonpresent SS",
							EXCEPTION_SS,(n_ss & 0xfffc)+(type&CPU_INT_SOFTWARE)?0:1)

						// commit point
						Segs.phys[ss]=n_ss_desc.GetBase();
						Segs.val[ss]=n_ss;
						if (n_ss_desc.Big()) {
							cpu.stack.big=true;
							cpu.stack.mask=0xffffffff;
							cpu.stack.notmask=0;
							reg_esp=n_esp;
						} else {
							cpu.stack.big=false;
							cpu.stack.mask=0xffff;
							cpu.stack.notmask=0xffff0000;
							reg_sp=n_esp & 0xffff;
						}

						cpu.cpl=cs_dpl;
						if (gate.Type() & 0x8) {	/* 32-bit Gate */
							if (reg_flags & FLAG_VM) {
								CPU_Push32(SegValue(gs));SegSet16(gs,0x0);
								CPU_Push32(SegValue(fs));SegSet16(fs,0x0);
								CPU_Push32(SegValue(ds));SegSet16(ds,0x0);
								CPU_Push32(SegValue(es));SegSet16(es,0x0);
							}
							CPU_Push32(o_ss);
							CPU_Push32(o_esp);
						} else {					/* 16-bit Gate */
							if (reg_flags & FLAG_VM) E_Exit("V86 to 16-bit gate");
							CPU_Push16(o_ss);
							CPU_Push16(o_esp);
						}
//						LOG_MSG("INT:Gate to inner level SS:%X SP:%X",n_ss,n_esp);
						goto do_interrupt;
					}
					if (cs_dpl!=cpu.cpl)
						E_Exit("Non-conforming intra privilege INT with DPL!=CPL");
				case DESC_CODE_N_C_A:	case DESC_CODE_N_C_NA:
				case DESC_CODE_R_C_A:	case DESC_CODE_R_C_NA:
					/* Prepare stack for gate to same priviledge */
					CPU_CHECK_COND(!cs_desc.saved.seg.p,
							"INT:Same level:CS segment not present",
						EXCEPTION_NP,(gate_sel & 0xfffc)+(type&CPU_INT_SOFTWARE)?0:1)
					if ((reg_flags & FLAG_VM) && (cs_dpl<cpu.cpl))
						E_Exit("V86 interrupt doesn't change to pl0");	// or #GP(cs_sel)

					// commit point
do_interrupt:
					if (gate.Type() & 0x8) {	/* 32-bit Gate */
						CPU_Push32(reg_flags);
						CPU_Push32(SegValue(cs));
						CPU_Push32(oldeip);
						if (type & CPU_INT_HAS_ERROR) CPU_Push32(cpu.exception.error);
					} else {					/* 16-bit gate */
						CPU_Push16(reg_flags & 0xffff);
						CPU_Push16(SegValue(cs));
						CPU_Push16(oldeip);
						if (type & CPU_INT_HAS_ERROR) CPU_Push16(cpu.exception.error);
					}
					break;
				default:
					E_Exit("INT:Gate Selector points to illegal descriptor with type %x",cs_desc.Type());
				}

				Segs.val[cs]=(gate_sel&0xfffc) | cpu.cpl;
				Segs.phys[cs]=cs_desc.GetBase();
				cpu.code.big=cs_desc.Big()>0;
				reg_eip=gate_off;

				if (!(gate.Type()&1)) {
					SETFLAGBIT(IF,false);
				}
				SETFLAGBIT(TF,false);
				SETFLAGBIT(NT,false);
				SETFLAGBIT(VM,false);
				LOG(LOG_CPU,LOG_NORMAL)("INT:Gate to %X:%X big %d %s",gate_sel,gate_off,cs_desc.Big(),gate.Type() & 0x8 ? "386" : "286");
				return;
			}
		case DESC_TASK_GATE:
			CPU_CHECK_COND(!gate.saved.seg.p,
				"INT:Gate segment not present",
				EXCEPTION_NP,num*8+2+(type&CPU_INT_SOFTWARE)?0:1)

			CPU_SwitchTask(gate.GetSelector(),TSwitch_CALL_INT,oldeip);
			if (type & CPU_INT_HAS_ERROR) {
				//TODO Be sure about this, seems somewhat unclear
				if (cpu_tss.is386) CPU_Push32(cpu.exception.error);
				else CPU_Push16(cpu.exception.error);
			}
			return;
		default:
			E_Exit("Illegal descriptor type %X for int %X",gate.Type(),num);
		}
	}
	assert(1);
	return ; // make compiler happy
}


void CPU_IRET(bool use32,Bitu oldeip) {
	if (!cpu.pmode) {					/* RealMode IRET */
		if (use32) {
			reg_eip=CPU_Pop32();
			SegSet16(cs,CPU_Pop32());
			CPU_SetFlags(CPU_Pop32(),FMASK_ALL);
		} else {
			reg_eip=CPU_Pop16();
			SegSet16(cs,CPU_Pop16());
			CPU_SetFlags(CPU_Pop16(),FMASK_ALL & 0xffff);
		}
		cpu.code.big=false;
		DestroyConditionFlags();
		return;
	} else {	/* Protected mode IRET */
		if (reg_flags & FLAG_VM) {
			if ((reg_flags & FLAG_IOPL)!=FLAG_IOPL) {
				// win3.x e
				CPU_Exception(EXCEPTION_GP,0);
				return;
			} else {
				if (use32) {
					Bit32u new_eip=mem_readd(SegPhys(ss) + (reg_esp & cpu.stack.mask));
					Bit32u tempesp=(reg_esp&cpu.stack.notmask)|((reg_esp+4)&cpu.stack.mask);
					Bit32u new_cs=mem_readd(SegPhys(ss) + (tempesp & cpu.stack.mask));
					tempesp=(tempesp&cpu.stack.notmask)|((tempesp+4)&cpu.stack.mask);
					Bit32u new_flags=mem_readd(SegPhys(ss) + (tempesp & cpu.stack.mask));
					reg_esp=(tempesp&cpu.stack.notmask)|((tempesp+4)&cpu.stack.mask);

					reg_eip=new_eip;
					SegSet16(cs,(Bit16u)(new_cs&0xffff));
					/* IOPL can not be modified in v86 mode by IRET */
					CPU_SetFlags(new_flags,FMASK_NORMAL|FLAG_NT);
				} else {
					Bit16u new_eip=mem_readw(SegPhys(ss) + (reg_esp & cpu.stack.mask));
					Bit32u tempesp=(reg_esp&cpu.stack.notmask)|((reg_esp+2)&cpu.stack.mask);
					Bit16u new_cs=mem_readw(SegPhys(ss) + (tempesp & cpu.stack.mask));
					tempesp=(tempesp&cpu.stack.notmask)|((tempesp+2)&cpu.stack.mask);
					Bit16u new_flags=mem_readw(SegPhys(ss) + (tempesp & cpu.stack.mask));
					reg_esp=(tempesp&cpu.stack.notmask)|((tempesp+2)&cpu.stack.mask);

					reg_eip=(Bit32u)new_eip;
					SegSet16(cs,new_cs);
					/* IOPL can not be modified in v86 mode by IRET */
					CPU_SetFlags(new_flags,FMASK_NORMAL|FLAG_NT);
				}
				cpu.code.big=false;
				DestroyConditionFlags();
				return;
			}
		}
		/* Check if this is task IRET */
		if (GETFLAG(NT)) {
			if (GETFLAG(VM)) E_Exit("Pmode IRET with VM bit set");
			CPU_CHECK_COND(!cpu_tss.IsValid(),
				"TASK Iret without valid TSS",
				EXCEPTION_TS,cpu_tss.selector & 0xfffc)
			if (!cpu_tss.desc.IsBusy()) {
				LOG(LOG_CPU,LOG_ERROR)("TASK Iret:TSS not busy");
			}
			Bitu back_link=cpu_tss.Get_back();
			CPU_SwitchTask(back_link,TSwitch_IRET,oldeip);
			return;
		}
		Bitu n_cs_sel,n_eip,n_flags;
		Bit32u tempesp;
		if (use32) {
			n_eip=mem_readd(SegPhys(ss) + (reg_esp & cpu.stack.mask));
			tempesp=(reg_esp&cpu.stack.notmask)|((reg_esp+4)&cpu.stack.mask);
			n_cs_sel=mem_readd(SegPhys(ss) + (tempesp & cpu.stack.mask)) & 0xffff;
			tempesp=(tempesp&cpu.stack.notmask)|((tempesp+4)&cpu.stack.mask);
			n_flags=mem_readd(SegPhys(ss) + (tempesp & cpu.stack.mask));
			tempesp=(tempesp&cpu.stack.notmask)|((tempesp+4)&cpu.stack.mask);

			if ((n_flags & FLAG_VM) && (cpu.cpl==0)) {
				// commit point
				reg_esp=tempesp;
				reg_eip=n_eip & 0xffff;
				Bitu n_ss,n_esp,n_es,n_ds,n_fs,n_gs;
				n_esp=CPU_Pop32();
				n_ss=CPU_Pop32() & 0xffff;
				n_es=CPU_Pop32() & 0xffff;
				n_ds=CPU_Pop32() & 0xffff;
				n_fs=CPU_Pop32() & 0xffff;
				n_gs=CPU_Pop32() & 0xffff;

				CPU_SetFlags(n_flags,FMASK_ALL | FLAG_VM);
				DestroyConditionFlags();
				cpu.cpl=3;

				CPU_SetSegGeneral(ss,n_ss);
				CPU_SetSegGeneral(es,n_es);
				CPU_SetSegGeneral(ds,n_ds);
				CPU_SetSegGeneral(fs,n_fs);
				CPU_SetSegGeneral(gs,n_gs);
				reg_esp=n_esp;
				cpu.code.big=false;
				SegSet16(cs,n_cs_sel);
				LOG(LOG_CPU,LOG_NORMAL)("IRET:Back to V86: CS:%X IP %X SS:%X SP %X FLAGS:%X",SegValue(cs),reg_eip,SegValue(ss),reg_esp,reg_flags);
				return;
			}
			if (n_flags & FLAG_VM) E_Exit("IRET from pmode to v86 with CPL!=0");
		} else {
			n_eip=mem_readw(SegPhys(ss) + (reg_esp & cpu.stack.mask));
			tempesp=(reg_esp&cpu.stack.notmask)|((reg_esp+2)&cpu.stack.mask);
			n_cs_sel=mem_readw(SegPhys(ss) + (tempesp & cpu.stack.mask));
			tempesp=(tempesp&cpu.stack.notmask)|((tempesp+2)&cpu.stack.mask);
			n_flags=mem_readw(SegPhys(ss) + (tempesp & cpu.stack.mask));
			n_flags|=(reg_flags & 0xffff0000);
			tempesp=(tempesp&cpu.stack.notmask)|((tempesp+2)&cpu.stack.mask);

			if (n_flags & FLAG_VM) E_Exit("VM Flag in 16-bit iret");
		}
		CPU_CHECK_COND((n_cs_sel & 0xfffc)==0,
			"IRET:CS selector zero",
			EXCEPTION_GP,0)
		Bitu n_cs_rpl=n_cs_sel & 3;
		Descriptor n_cs_desc;
		CPU_CHECK_COND(!cpu.gdt.GetDescriptor(n_cs_sel,n_cs_desc),
			"IRET:CS selector beyond limits",
			EXCEPTION_GP,n_cs_sel & 0xfffc)
		CPU_CHECK_COND(n_cs_rpl<cpu.cpl,
			"IRET to lower privilege",
			EXCEPTION_GP,n_cs_sel & 0xfffc)

		switch (n_cs_desc.Type()) {
		case DESC_CODE_N_NC_A:	case DESC_CODE_N_NC_NA:
		case DESC_CODE_R_NC_A:	case DESC_CODE_R_NC_NA:
			CPU_CHECK_COND(n_cs_rpl!=n_cs_desc.DPL(),
				"IRET:NC:DPL!=RPL",
				EXCEPTION_GP,n_cs_sel & 0xfffc)
			break;
		case DESC_CODE_N_C_A:	case DESC_CODE_N_C_NA:
		case DESC_CODE_R_C_A:	case DESC_CODE_R_C_NA:
			CPU_CHECK_COND(n_cs_desc.DPL()>n_cs_rpl,
				"IRET:C:DPL>RPL",
				EXCEPTION_GP,n_cs_sel & 0xfffc)
			break;
		default:
			E_Exit("IRET:Illegal descriptor type %X",n_cs_desc.Type());
		}
		CPU_CHECK_COND(!n_cs_desc.saved.seg.p,
			"IRET with nonpresent code segment",
			EXCEPTION_NP,n_cs_sel & 0xfffc)

		if (n_cs_rpl==cpu.cpl) {
			/* Return to same level */

			// commit point
			reg_esp=tempesp;
			Segs.phys[cs]=n_cs_desc.GetBase();
			cpu.code.big=n_cs_desc.Big()>0;
			Segs.val[cs]=n_cs_sel;
			reg_eip=n_eip;

			Bitu mask=cpu.cpl ? (FMASK_NORMAL | FLAG_NT) : FMASK_ALL;
			if (GETFLAG_IOPL<cpu.cpl) mask &= (~FLAG_IF);
			CPU_SetFlags(n_flags,mask);
			DestroyConditionFlags();
			LOG(LOG_CPU,LOG_NORMAL)("IRET:Same level:%X:%X big %d",n_cs_sel,n_eip,cpu.code.big);
		} else {
			/* Return to outer level */
			Bitu n_ss,n_esp;
			if (use32) {
				n_esp=mem_readd(SegPhys(ss) + (tempesp & cpu.stack.mask));
				tempesp=(tempesp&cpu.stack.notmask)|((tempesp+4)&cpu.stack.mask);
				n_ss=mem_readd(SegPhys(ss) + (tempesp & cpu.stack.mask)) & 0xffff;
			} else {
				n_esp=mem_readw(SegPhys(ss) + (tempesp & cpu.stack.mask));
				tempesp=(tempesp&cpu.stack.notmask)|((tempesp+2)&cpu.stack.mask);
				n_ss=mem_readw(SegPhys(ss) + (tempesp & cpu.stack.mask));
			}
			CPU_CHECK_COND((n_ss & 0xfffc)==0,
				"IRET:Outer level:SS selector zero",
				EXCEPTION_GP,0)
			CPU_CHECK_COND((n_ss & 3)!=n_cs_rpl,
				"IRET:Outer level:SS rpl!=CS rpl",
				EXCEPTION_GP,n_ss & 0xfffc)
			Descriptor n_ss_desc;
			CPU_CHECK_COND(!cpu.gdt.GetDescriptor(n_ss,n_ss_desc),
				"IRET:Outer level:SS beyond limit",
				EXCEPTION_GP,n_ss & 0xfffc)
			CPU_CHECK_COND(n_ss_desc.DPL()!=n_cs_rpl,
				"IRET:Outer level:SS dpl!=CS rpl",
				EXCEPTION_GP,n_ss & 0xfffc)

			// check if stack segment is a writable data segment
			switch (n_ss_desc.Type()) {
			case DESC_DATA_EU_RW_NA:		case DESC_DATA_EU_RW_A:
			case DESC_DATA_ED_RW_NA:		case DESC_DATA_ED_RW_A:
				break;
			default:
				E_Exit("IRET:Outer level:Stack segment not writable");		// or #GP(ss_sel)
			}
			CPU_CHECK_COND(!n_ss_desc.saved.seg.p,
				"IRET:Outer level:Stack segment not present",
				EXCEPTION_NP,n_ss & 0xfffc)

			// commit point

			Segs.phys[cs]=n_cs_desc.GetBase();
			cpu.code.big=n_cs_desc.Big()>0;
			Segs.val[cs]=n_cs_sel;

			Bitu mask=cpu.cpl ? (FMASK_NORMAL | FLAG_NT) : FMASK_ALL;
			if (GETFLAG_IOPL<cpu.cpl) mask &= (~FLAG_IF);
			CPU_SetFlags(n_flags,mask);
			DestroyConditionFlags();

			cpu.cpl=n_cs_rpl;
			reg_eip=n_eip;

			Segs.val[ss]=n_ss;
			Segs.phys[ss]=n_ss_desc.GetBase();
			if (n_ss_desc.Big()) {
				cpu.stack.big=true;
				cpu.stack.mask=0xffffffff;
				cpu.stack.notmask=0;
				reg_esp=n_esp;
			} else {
				cpu.stack.big=false;
				cpu.stack.mask=0xffff;
				cpu.stack.notmask=0xffff0000;
				reg_sp=n_esp & 0xffff;
			}

			// borland extender, zrdx
			CPU_CheckSegments();

			LOG(LOG_CPU,LOG_NORMAL)("IRET:Outer level:%X:%X big %d",n_cs_sel,n_eip,cpu.code.big);
		}
		return;
	}
}


void CPU_JMP(bool use32,Bitu selector,Bitu offset,Bitu oldeip) {
	if (!cpu.pmode || (reg_flags & FLAG_VM)) {
		if (!use32) {
			reg_eip=offset&0xffff;
		} else {
			reg_eip=offset;
		}
		SegSet16(cs,selector);
		cpu.code.big=false;
		return;
	} else {
		CPU_CHECK_COND((selector & 0xfffc)==0,
			"JMP:CS selector zero",
			EXCEPTION_GP,0)
		Bitu rpl=selector & 3;
		Descriptor desc;
		CPU_CHECK_COND(!cpu.gdt.GetDescriptor(selector,desc),
			"JMP:CS beyond limits",
			EXCEPTION_GP,selector & 0xfffc)
		switch (desc.Type()) {
		case DESC_CODE_N_NC_A:		case DESC_CODE_N_NC_NA:
		case DESC_CODE_R_NC_A:		case DESC_CODE_R_NC_NA:
			CPU_CHECK_COND(rpl>cpu.cpl,
				"JMP:NC:RPL>CPL",
				EXCEPTION_GP,selector & 0xfffc)
			CPU_CHECK_COND(cpu.cpl!=desc.DPL(),
				"JMP:NC:RPL != DPL",
				EXCEPTION_GP,selector & 0xfffc)
			LOG(LOG_CPU,LOG_NORMAL)("JMP:Code:NC to %X:%X big %d",selector,offset,desc.Big());
			goto CODE_jmp;
		case DESC_CODE_N_C_A:		case DESC_CODE_N_C_NA:
		case DESC_CODE_R_C_A:		case DESC_CODE_R_C_NA:
			LOG(LOG_CPU,LOG_NORMAL)("JMP:Code:C to %X:%X big %d",selector,offset,desc.Big());
			CPU_CHECK_COND(cpu.cpl<desc.DPL(),
				"JMP:C:CPL < DPL",
				EXCEPTION_GP,selector & 0xfffc)
CODE_jmp:
			if (!desc.saved.seg.p) {
				// win
				CPU_Exception(EXCEPTION_NP,selector & 0xfffc);
				return;
			}

			/* Normal jump to another selector:offset */
			Segs.phys[cs]=desc.GetBase();
			cpu.code.big=desc.Big()>0;
			Segs.val[cs]=(selector & 0xfffc) | cpu.cpl;
			reg_eip=offset;
			return;
		case DESC_386_TSS_A:
			CPU_CHECK_COND(desc.DPL()<cpu.cpl,
				"JMP:TSS:dpl<cpl",
				EXCEPTION_GP,selector & 0xfffc)
			CPU_CHECK_COND(desc.DPL()<rpl,
				"JMP:TSS:dpl<rpl",
				EXCEPTION_GP,selector & 0xfffc)
			LOG(LOG_CPU,LOG_NORMAL)("JMP:TSS to %X",selector);
			CPU_SwitchTask(selector,TSwitch_JMP,oldeip);
			break;
		default:
			E_Exit("JMP Illegal descriptor type %X",desc.Type());
		}
	}
	assert(1);
}


void CPU_CALL(bool use32,Bitu selector,Bitu offset,Bitu oldeip) {
	if (!cpu.pmode || (reg_flags & FLAG_VM)) {
		if (!use32) {
			CPU_Push16(SegValue(cs));
			CPU_Push16(oldeip);
			reg_eip=offset&0xffff;
		} else {
			CPU_Push32(SegValue(cs));
			CPU_Push32(oldeip);
			reg_eip=offset;
		}
		cpu.code.big=false;
		SegSet16(cs,selector);
		return;
	} else {
		CPU_CHECK_COND((selector & 0xfffc)==0,
			"CALL:CS selector zero",
			EXCEPTION_GP,0)
		Bitu rpl=selector & 3;
		Descriptor call;
		CPU_CHECK_COND(!cpu.gdt.GetDescriptor(selector,call),
			"CALL:CS beyond limits",
			EXCEPTION_GP,selector & 0xfffc)
		/* Check for type of far call */
		switch (call.Type()) {
		case DESC_CODE_N_NC_A:case DESC_CODE_N_NC_NA:
		case DESC_CODE_R_NC_A:case DESC_CODE_R_NC_NA:
			CPU_CHECK_COND(rpl>cpu.cpl,
				"CALL:CODE:NC:RPL>CPL",
				EXCEPTION_GP,selector & 0xfffc)
			CPU_CHECK_COND(call.DPL()!=cpu.cpl,
				"CALL:CODE:NC:DPL!=CPL",
				EXCEPTION_GP,selector & 0xfffc)
			LOG(LOG_CPU,LOG_NORMAL)("CALL:CODE:NC to %X:%X",selector,offset);
			goto call_code;
		case DESC_CODE_N_C_A:case DESC_CODE_N_C_NA:
		case DESC_CODE_R_C_A:case DESC_CODE_R_C_NA:
			CPU_CHECK_COND(call.DPL()>cpu.cpl,
				"CALL:CODE:C:DPL>CPL",
				EXCEPTION_GP,selector & 0xfffc)
			LOG(LOG_CPU,LOG_NORMAL)("CALL:CODE:C to %X:%X",selector,offset);
call_code:
			if (!call.saved.seg.p) {
				// borland extender (RTM)
				CPU_Exception(EXCEPTION_NP,selector & 0xfffc);
				return;
			}
			// commit point
			if (!use32) {
				CPU_Push16(SegValue(cs));
				CPU_Push16(oldeip);
				reg_eip=offset & 0xffff;
			} else {
				CPU_Push32(SegValue(cs));
				CPU_Push32(oldeip);
				reg_eip=offset;
			}
			Segs.phys[cs]=call.GetBase();
			cpu.code.big=call.Big()>0;
			Segs.val[cs]=(selector & 0xfffc) | cpu.cpl;
			return;
		case DESC_386_CALL_GATE:
		case DESC_286_CALL_GATE:
			{
				CPU_CHECK_COND(call.DPL()<cpu.cpl,
					"CALL:Gate:Gate DPL<CPL",
					EXCEPTION_GP,selector & 0xfffc)
				CPU_CHECK_COND(call.DPL()<rpl,
					"CALL:Gate:Gate DPL<RPL",
					EXCEPTION_GP,selector & 0xfffc)
				CPU_CHECK_COND(!call.saved.seg.p,
					"CALL:Gate:Segment not present",
					EXCEPTION_NP,selector & 0xfffc)
				Descriptor n_cs_desc;
				Bitu n_cs_sel=call.GetSelector();

				CPU_CHECK_COND((n_cs_sel & 0xfffc)==0,
					"CALL:Gate:CS selector zero",
					EXCEPTION_GP,0)
				CPU_CHECK_COND(!cpu.gdt.GetDescriptor(n_cs_sel,n_cs_desc),
					"CALL:Gate:CS beyond limits",
					EXCEPTION_GP,n_cs_sel & 0xfffc)
				Bitu n_cs_dpl	= n_cs_desc.DPL();
				CPU_CHECK_COND(n_cs_dpl>cpu.cpl,
					"CALL:Gate:CS DPL>CPL",
					EXCEPTION_GP,n_cs_sel & 0xfffc)

				CPU_CHECK_COND(!n_cs_desc.saved.seg.p,
					"CALL:Gate:CS not present",
					EXCEPTION_NP,n_cs_sel & 0xfffc)

				Bitu n_eip		= call.GetOffset();
				switch (n_cs_desc.Type()) {
				case DESC_CODE_N_NC_A:case DESC_CODE_N_NC_NA:
				case DESC_CODE_R_NC_A:case DESC_CODE_R_NC_NA:
					/* Check if we goto inner priviledge */
					if (n_cs_dpl < cpu.cpl) {
						/* Get new SS:ESP out of TSS */
						Bitu n_ss_sel,n_esp;
						Descriptor n_ss_desc;
						cpu_tss.Get_SSx_ESPx(n_cs_dpl,n_ss_sel,n_esp);
						CPU_CHECK_COND((n_ss_sel & 0xfffc)==0,
							"CALL:Gate:NC:SS selector zero",
							EXCEPTION_TS,0)
						CPU_CHECK_COND(!cpu.gdt.GetDescriptor(n_ss_sel,n_ss_desc),
							"CALL:Gate:Invalid SS selector",
							EXCEPTION_TS,n_ss_sel & 0xfffc)
						CPU_CHECK_COND(((n_ss_sel & 3)!=n_cs_desc.DPL()) || (n_ss_desc.DPL()!=n_cs_desc.DPL()),
							"CALL:Gate:Invalid SS selector privileges",
							EXCEPTION_TS,n_ss_sel & 0xfffc)

						switch (n_ss_desc.Type()) {
						case DESC_DATA_EU_RW_NA:		case DESC_DATA_EU_RW_A:
						case DESC_DATA_ED_RW_NA:		case DESC_DATA_ED_RW_A:
							// writable data segment
							break;
						default:
							E_Exit("Call:Gate:SS no writable data segment");	// or #TS(ss_sel)
						}
						CPU_CHECK_COND(!n_ss_desc.saved.seg.p,
							"CALL:Gate:Stack segment not present",
							EXCEPTION_SS,n_ss_sel & 0xfffc)

						/* Load the new SS:ESP and save data on it */
						Bitu o_esp		= reg_esp;
						Bitu o_ss		= SegValue(ss);
						PhysPt o_stack  = SegPhys(ss)+(reg_esp & cpu.stack.mask);


						// catch pagefaults
						if (call.saved.gate.paramcount&31) {
							if (call.Type()==DESC_386_CALL_GATE) {
								for (Bits i=(call.saved.gate.paramcount&31)-1;i>=0;i--)
									mem_readd(o_stack+i*4);
							} else {
								for (Bits i=(call.saved.gate.paramcount&31)-1;i>=0;i--)
									mem_readw(o_stack+i*2);
							}
						}

						// commit point
						Segs.val[ss]=n_ss_sel;
						Segs.phys[ss]=n_ss_desc.GetBase();
						if (n_ss_desc.Big()) {
							cpu.stack.big=true;
							cpu.stack.mask=0xffffffff;
							cpu.stack.notmask=0;
							reg_esp=n_esp;
						} else {
							cpu.stack.big=false;
							cpu.stack.mask=0xffff;
							cpu.stack.notmask=0xffff0000;
							reg_sp=n_esp & 0xffff;
						}

						cpu.cpl = n_cs_desc.DPL();
						Bit16u oldcs    = SegValue(cs);
						/* Switch to new CS:EIP */
						Segs.phys[cs]	= n_cs_desc.GetBase();
						Segs.val[cs]	= (n_cs_sel & 0xfffc) | cpu.cpl;
						cpu.code.big	= n_cs_desc.Big()>0;
						reg_eip			= n_eip;
						if (!use32)	reg_eip&=0xffff;

						if (call.Type()==DESC_386_CALL_GATE) {
							CPU_Push32(o_ss);		//save old stack
							CPU_Push32(o_esp);
							if (call.saved.gate.paramcount&31)
								for (Bits i=(call.saved.gate.paramcount&31)-1;i>=0;i--)
									CPU_Push32(mem_readd(o_stack+i*4));
							CPU_Push32(oldcs);
							CPU_Push32(oldeip);
						} else {
							CPU_Push16(o_ss);		//save old stack
							CPU_Push16(o_esp);
							if (call.saved.gate.paramcount&31)
								for (Bits i=(call.saved.gate.paramcount&31)-1;i>=0;i--)
									CPU_Push16(mem_readw(o_stack+i*2));
							CPU_Push16(oldcs);
							CPU_Push16(oldeip);
						}

						break;
					} else if (n_cs_dpl > cpu.cpl)
						E_Exit("CALL:GATE:CS DPL>CPL");		// or #GP(sel)
				case DESC_CODE_N_C_A:case DESC_CODE_N_C_NA:
				case DESC_CODE_R_C_A:case DESC_CODE_R_C_NA:
					// zrdx extender

					if (call.Type()==DESC_386_CALL_GATE) {
						CPU_Push32(SegValue(cs));
						CPU_Push32(oldeip);
					} else {
						CPU_Push16(SegValue(cs));
						CPU_Push16(oldeip);
					}

					/* Switch to new CS:EIP */
					Segs.phys[cs]	= n_cs_desc.GetBase();
					Segs.val[cs]	= (n_cs_sel & 0xfffc) | cpu.cpl;
					cpu.code.big	= n_cs_desc.Big()>0;
					reg_eip			= n_eip;
					if (!use32)	reg_eip&=0xffff;
					break;
				default:
					E_Exit("CALL:GATE:CS no executable segment");
				}
			}			/* Call Gates */
			break;
		case DESC_386_TSS_A:
			CPU_CHECK_COND(call.DPL()<cpu.cpl,
				"CALL:TSS:dpl<cpl",
				EXCEPTION_GP,selector & 0xfffc)
			CPU_CHECK_COND(call.DPL()<rpl,
				"CALL:TSS:dpl<rpl",
				EXCEPTION_GP,selector & 0xfffc)

			CPU_CHECK_COND(!call.saved.seg.p,
				"CALL:TSS:Segment not present",
				EXCEPTION_NP,selector & 0xfffc)

			LOG(LOG_CPU,LOG_NORMAL)("CALL:TSS to %X",selector);
			CPU_SwitchTask(selector,TSwitch_CALL_INT,oldeip);
			break;
		case DESC_DATA_EU_RW_NA:	// vbdos
		case DESC_INVALID:			// used by some installers
			CPU_Exception(EXCEPTION_GP,selector & 0xfffc);
			return;
		default:
			E_Exit("CALL:Descriptor type %x unsupported",call.Type());
		}
	}
	assert(1);
}


void CPU_RET(bool use32,Bitu bytes,Bitu oldeip) {
	if (!cpu.pmode || (reg_flags & FLAG_VM)) {
		Bitu new_ip,new_cs;
		if (!use32) {
			new_ip=CPU_Pop16();
			new_cs=CPU_Pop16();
		} else {
			new_ip=CPU_Pop32();
			new_cs=CPU_Pop32() & 0xffff;
		}
		reg_esp+=bytes;
		SegSet16(cs,new_cs);
		reg_eip=new_ip;
		cpu.code.big=false;
		return;
	} else {
		Bitu offset,selector;
		if (!use32) selector	= mem_readw(SegPhys(ss) + (reg_esp & cpu.stack.mask) + 2);
		else 		selector	= mem_readd(SegPhys(ss) + (reg_esp & cpu.stack.mask) + 4) & 0xffff;

		Descriptor desc;
		Bitu rpl=selector & 3;
		if(rpl < cpu.cpl) {
			// win setup
			CPU_Exception(EXCEPTION_GP,selector & 0xfffc);
			return;
		}

		CPU_CHECK_COND((selector & 0xfffc)==0,
			"RET:CS selector zero",
			EXCEPTION_GP,0)
		CPU_CHECK_COND(!cpu.gdt.GetDescriptor(selector,desc),
			"RET:CS beyond limits",
			EXCEPTION_GP,selector & 0xfffc)

		if (cpu.cpl==rpl) {
			/* Return to same level */
			switch (desc.Type()) {
			case DESC_CODE_N_NC_A:case DESC_CODE_N_NC_NA:
			case DESC_CODE_R_NC_A:case DESC_CODE_R_NC_NA:
				CPU_CHECK_COND(cpu.cpl!=desc.DPL(),
					"RET to NC segment of other privilege",
					EXCEPTION_GP,selector & 0xfffc)
				goto RET_same_level;
			case DESC_CODE_N_C_A:case DESC_CODE_N_C_NA:
			case DESC_CODE_R_C_A:case DESC_CODE_R_C_NA:
				CPU_CHECK_COND(desc.DPL()>cpu.cpl,
					"RET to C segment of higher privilege",
					EXCEPTION_GP,selector & 0xfffc)
				break;
			default:
				E_Exit("RET from illegal descriptor type %X",desc.Type());
			}
RET_same_level:
			if (!desc.saved.seg.p) {
				// borland extender (RTM)
				CPU_Exception(EXCEPTION_NP,selector & 0xfffc);
				return;
			}

			// commit point
			if (!use32) {
				offset=CPU_Pop16();
				selector=CPU_Pop16();
			} else {
				offset=CPU_Pop32();
				selector=CPU_Pop32() & 0xffff;
			}

			Segs.phys[cs]=desc.GetBase();
			cpu.code.big=desc.Big()>0;
			Segs.val[cs]=selector;
			reg_eip=offset;
			if (cpu.stack.big) {
				reg_esp+=bytes;
			} else {
				reg_sp+=bytes;
			}
			LOG(LOG_CPU,LOG_NORMAL)("RET - Same level to %X:%X RPL %X DPL %X",selector,offset,rpl,desc.DPL());
			return;
		} else {
			/* Return to outer level */
			switch (desc.Type()) {
			case DESC_CODE_N_NC_A:case DESC_CODE_N_NC_NA:
			case DESC_CODE_R_NC_A:case DESC_CODE_R_NC_NA:
				CPU_CHECK_COND(desc.DPL()!=rpl,
					"RET to outer NC segment with DPL!=RPL",
					EXCEPTION_GP,selector & 0xfffc)
				break;
			case DESC_CODE_N_C_A:case DESC_CODE_N_C_NA:
			case DESC_CODE_R_C_A:case DESC_CODE_R_C_NA:
				CPU_CHECK_COND(desc.DPL()>rpl,
					"RET to outer C segment with DPL>RPL",
					EXCEPTION_GP,selector & 0xfffc)
				break;
			default:
				E_Exit("RET from illegal descriptor type %X",desc.Type());		// or #GP(selector)
			}

			CPU_CHECK_COND(!desc.saved.seg.p,
				"RET:Outer level:CS not present",
				EXCEPTION_NP,selector & 0xfffc)

			// commit point
			Bitu n_esp,n_ss;
			if (use32) {
				offset=CPU_Pop32();
				selector=CPU_Pop32() & 0xffff;
				reg_esp+=bytes;
				n_esp = CPU_Pop32();
				n_ss = CPU_Pop32() & 0xffff;
			} else {
				offset=CPU_Pop16();
				selector=CPU_Pop16();
				reg_esp+=bytes;
				n_esp = CPU_Pop16();
				n_ss = CPU_Pop16();
			}

			CPU_CHECK_COND((n_ss & 0xfffc)==0,
				"RET to outer level with SS selector zero",
				EXCEPTION_GP,0)

			Descriptor n_ss_desc;
			CPU_CHECK_COND(!cpu.gdt.GetDescriptor(n_ss,n_ss_desc),
				"RET:SS beyond limits",
				EXCEPTION_GP,n_ss & 0xfffc)

			CPU_CHECK_COND(((n_ss & 3)!=rpl) || (n_ss_desc.DPL()!=rpl),
				"RET to outer segment with invalid SS privileges",
				EXCEPTION_GP,n_ss & 0xfffc)
			switch (n_ss_desc.Type()) {
			case DESC_DATA_EU_RW_NA:		case DESC_DATA_EU_RW_A:
			case DESC_DATA_ED_RW_NA:		case DESC_DATA_ED_RW_A:
				break;
			default:
				E_Exit("RET:SS selector type no writable data segment");	// or #GP(selector)
			}
			CPU_CHECK_COND(!n_ss_desc.saved.seg.p,
				"RET:Stack segment not present",
				EXCEPTION_SS,n_ss & 0xfffc)

			cpu.cpl = rpl;
			Segs.phys[cs]=desc.GetBase();
			cpu.code.big=desc.Big()>0;
			Segs.val[cs]=(selector&0xfffc) | cpu.cpl;
			reg_eip=offset;

			Segs.val[ss]=n_ss;
			Segs.phys[ss]=n_ss_desc.GetBase();
			if (n_ss_desc.Big()) {
				cpu.stack.big=true;
				cpu.stack.mask=0xffffffff;
				cpu.stack.notmask=0;
				reg_esp=n_esp+bytes;
			} else {
				cpu.stack.big=false;
				cpu.stack.mask=0xffff;
				cpu.stack.notmask=0xffff0000;
				reg_sp=(n_esp & 0xffff)+bytes;
			}

			CPU_CheckSegments();

//			LOG(LOG_MISC,LOG_ERROR)("RET - Higher level to %X:%X RPL %X DPL %X",selector,offset,rpl,desc.DPL());
			return;
		}
		LOG(LOG_CPU,LOG_NORMAL)("Prot ret %X:%X",selector,offset);
		return;
	}
	assert(1);
}


Bitu CPU_SLDT(void) {
	return cpu.gdt.SLDT();
}

bool CPU_LLDT(Bitu selector) {
	if (!cpu.gdt.LLDT(selector)) {
		LOG(LOG_CPU,LOG_ERROR)("LLDT failed, selector=%X",selector);
		return true;
	}
	LOG(LOG_CPU,LOG_NORMAL)("LDT Set to %X",selector);
	return false;
}

Bitu CPU_STR(void) {
	return cpu_tss.selector;
}

bool CPU_LTR(Bitu selector) {
	if ((selector & 0xfffc)==0) {
		cpu_tss.SetSelector(selector);
		return false;
	}
	TSS_Descriptor desc;
	if ((selector & 4) || (!cpu.gdt.GetDescriptor(selector,desc))) {
		LOG(LOG_CPU,LOG_ERROR)("LTR failed, selector=%X",selector);
		return CPU_PrepareException(EXCEPTION_GP,selector);
	}

	if ((desc.Type()==DESC_286_TSS_A) || (desc.Type()==DESC_386_TSS_A)) {
		if (!desc.saved.seg.p) {
			LOG(LOG_CPU,LOG_ERROR)("LTR failed, selector=%X (not present)",selector);
			return CPU_PrepareException(EXCEPTION_NP,selector);
		}
		if (!cpu_tss.SetSelector(selector)) E_Exit("LTR failed, selector=%X",selector);
		cpu_tss.desc.SetBusy(true);
		cpu_tss.SaveSelector();
	} else {
		/* Descriptor was no available TSS descriptor */
		LOG(LOG_CPU,LOG_NORMAL)("LTR failed, selector=%X (type=%X)",selector,desc.Type());
		return CPU_PrepareException(EXCEPTION_GP,selector);
	}
	return false;
}

void CPU_LGDT(Bitu limit,Bitu base) {
	LOG(LOG_CPU,LOG_NORMAL)("GDT Set to base:%X limit:%X",base,limit);
	cpu.gdt.SetLimit(limit);
	cpu.gdt.SetBase(base);
}

void CPU_LIDT(Bitu limit,Bitu base) {
	LOG(LOG_CPU,LOG_NORMAL)("IDT Set to base:%X limit:%X",base,limit);
	cpu.idt.SetLimit(limit);
	cpu.idt.SetBase(base);
}

Bitu CPU_SGDT_base(void) {
	return cpu.gdt.GetBase();
}
Bitu CPU_SGDT_limit(void) {
	return cpu.gdt.GetLimit();
}

Bitu CPU_SIDT_base(void) {
	return cpu.idt.GetBase();
}
Bitu CPU_SIDT_limit(void) {
	return cpu.idt.GetLimit();
}

static bool printed_cycles_auto_info = false;
void CPU_SET_CRX(Bitu cr,Bitu value) {
	switch (cr) {
	case 0:
		{
			Bitu changed=cpu.cr0 ^ value;
			if (!changed) return;
			cpu.cr0=value;
			if (value & CR0_PROTECTION) {
				cpu.pmode=true;
				LOG(LOG_CPU,LOG_NORMAL)("Protected mode");
				PAGING_Enable((value & CR0_PAGING)>0);

				if (!(CPU_AutoDetermineMode&CPU_AUTODETERMINE_MASK)) break;

				if (CPU_AutoDetermineMode&CPU_AUTODETERMINE_CYCLES) {
					CPU_CycleAutoAdjust=true;
					CPU_CycleLeft=0;
					CPU_Cycles=0;
					CPU_OldCycleMax=CPU_CycleMax;
					GFX_SetTitle(CPU_CyclePercUsed,-1,false);
					if(!printed_cycles_auto_info) {
						printed_cycles_auto_info = true;
						LOG_MSG("DOSBox switched to max cycles, because of the setting: cycles=auto. If the game runs too fast try a fixed cycles amount in DOSBox's options.");
					}
				} else {
					GFX_SetTitle(-1,-1,false);
				}
#if (C_DYNAMIC_X86)
				if (CPU_AutoDetermineMode&CPU_AUTODETERMINE_CORE) {
					CPU_Core_Dyn_X86_Cache_Init(true);
					cpudecoder=&CPU_Core_Dyn_X86_Run;
				}
#elif (C_DYNREC)
				if (CPU_AutoDetermineMode&CPU_AUTODETERMINE_CORE) {
					CPU_Core_Dynrec_Cache_Init(true);
					cpudecoder=&CPU_Core_Dynrec_Run;
				}
#endif
				CPU_AutoDetermineMode<<=CPU_AUTODETERMINE_SHIFT;
			} else {
				cpu.pmode=false;
				if (value & CR0_PAGING) LOG_MSG("Paging requested without PE=1");
				PAGING_Enable(false);
				LOG(LOG_CPU,LOG_NORMAL)("Real mode");
			}
			break;
		}
	case 2:
		paging.cr2=value;
		break;
	case 3:
		PAGING_SetDirBase(value);
		break;
	default:
		LOG(LOG_CPU,LOG_ERROR)("Unhandled MOV CR%d,%X",cr,value);
		break;
	}
}

bool CPU_WRITE_CRX(Bitu cr,Bitu value) {
	/* Check if privileged to access control registers */
	if (cpu.pmode && (cpu.cpl>0)) return CPU_PrepareException(EXCEPTION_GP,0);
	if ((cr==1) || (cr>4)) return CPU_PrepareException(EXCEPTION_UD,0);
	if (CPU_ArchitectureType<CPU_ARCHTYPE_486OLDSLOW) {
		if (cr==4) return CPU_PrepareException(EXCEPTION_UD,0);
	}
	CPU_SET_CRX(cr,value);
	return false;
}

Bitu CPU_GET_CRX(Bitu cr) {
	switch (cr) {
	case 0:
		if (CPU_ArchitectureType>=CPU_ARCHTYPE_PENTIUMSLOW) return cpu.cr0;
		else if (CPU_ArchitectureType>=CPU_ARCHTYPE_486OLDSLOW) return (cpu.cr0 & 0xe005003f);
		else return (cpu.cr0 | 0x7ffffff0);
	case 2:
		return paging.cr2;
	case 3:
		return PAGING_GetDirBase() & 0xfffff000;
	default:
		LOG(LOG_CPU,LOG_ERROR)("Unhandled MOV XXX, CR%d",cr);
		break;
	}
	return 0;
}

bool CPU_READ_CRX(Bitu cr,Bit32u & retvalue) {
	/* Check if privileged to access control registers */
	if (cpu.pmode && (cpu.cpl>0)) return CPU_PrepareException(EXCEPTION_GP,0);
	if ((cr==1) || (cr>4)) return CPU_PrepareException(EXCEPTION_UD,0);
	retvalue=CPU_GET_CRX(cr);
	return false;
}


bool CPU_WRITE_DRX(Bitu dr,Bitu value) {
	/* Check if privileged to access control registers */
	if (cpu.pmode && (cpu.cpl>0)) return CPU_PrepareException(EXCEPTION_GP,0);
	switch (dr) {
	case 0:
	case 1:
	case 2:
	case 3:
		cpu.drx[dr]=value;
		break;
	case 4:
	case 6:
		cpu.drx[6]=(value|0xffff0ff0) & 0xffffefff;
		break;
	case 5:
	case 7:
		if (CPU_ArchitectureType<CPU_ARCHTYPE_PENTIUMSLOW) {
			cpu.drx[7]=(value|0x400) & 0xffff2fff;
		} else {
			cpu.drx[7]=(value|0x400);
		}
		break;
	default:
		LOG(LOG_CPU,LOG_ERROR)("Unhandled MOV DR%d,%X",dr,value);
		break;
	}
	return false;
}

bool CPU_READ_DRX(Bitu dr,Bit32u & retvalue) {
	/* Check if privileged to access control registers */
	if (cpu.pmode && (cpu.cpl>0)) return CPU_PrepareException(EXCEPTION_GP,0);
	switch (dr) {
	case 0:
	case 1:
	case 2:
	case 3:
	case 6:
	case 7:
		retvalue=cpu.drx[dr];
		break;
	case 4:
		retvalue=cpu.drx[6];
		break;
	case 5:
		retvalue=cpu.drx[7];
		break;
	default:
		LOG(LOG_CPU,LOG_ERROR)("Unhandled MOV XXX, DR%d",dr);
		retvalue=0;
		break;
	}
	return false;
}

bool CPU_WRITE_TRX(Bitu tr,Bitu value) {
	/* Check if privileged to access control registers */
	if (cpu.pmode && (cpu.cpl>0)) return CPU_PrepareException(EXCEPTION_GP,0);
	switch (tr) {
//	case 3:
	case 6:
	case 7:
		cpu.trx[tr]=value;
		return false;
	default:
		LOG(LOG_CPU,LOG_ERROR)("Unhandled MOV TR%d,%X",tr,value);
		break;
	}
	return CPU_PrepareException(EXCEPTION_UD,0);
}

bool CPU_READ_TRX(Bitu tr,Bit32u & retvalue) {
	/* Check if privileged to access control registers */
	if (cpu.pmode && (cpu.cpl>0)) return CPU_PrepareException(EXCEPTION_GP,0);
	switch (tr) {
//	case 3:
	case 6:
	case 7:
		retvalue=cpu.trx[tr];
		return false;
	default:
		LOG(LOG_CPU,LOG_ERROR)("Unhandled MOV XXX, TR%d",tr);
		break;
	}
	return CPU_PrepareException(EXCEPTION_UD,0);
}


Bitu CPU_SMSW(void) {
	return cpu.cr0;
}

bool CPU_LMSW(Bitu word) {
	if (cpu.pmode && (cpu.cpl>0)) return CPU_PrepareException(EXCEPTION_GP,0);
	word&=0xf;
	if (cpu.cr0 & 1) word|=1;
	word|=(cpu.cr0&0xfffffff0);
	CPU_SET_CRX(0,word);
	return false;
}

void CPU_ARPL(Bitu & dest_sel,Bitu src_sel) {
	FillFlags();
	if ((dest_sel & 3) < (src_sel & 3)) {
		dest_sel=(dest_sel & 0xfffc) + (src_sel & 3);
//		dest_sel|=0xff3f0000;
		SETFLAGBIT(ZF,true);
	} else {
		SETFLAGBIT(ZF,false);
	}
}

void CPU_LAR(Bitu selector,Bitu & ar) {
	FillFlags();
	if (selector == 0) {
		SETFLAGBIT(ZF,false);
		return;
	}
	Descriptor desc;Bitu rpl=selector & 3;
	if (!cpu.gdt.GetDescriptor(selector,desc)){
		SETFLAGBIT(ZF,false);
		return;
	}
	switch (desc.Type()){
	case DESC_CODE_N_C_A:	case DESC_CODE_N_C_NA:
	case DESC_CODE_R_C_A:	case DESC_CODE_R_C_NA:
		break;

	case DESC_286_INT_GATE:		case DESC_286_TRAP_GATE:	{
	case DESC_386_INT_GATE:		case DESC_386_TRAP_GATE:
		SETFLAGBIT(ZF,false);
		return;
	}

	case DESC_LDT:
	case DESC_TASK_GATE:

	case DESC_286_TSS_A:		case DESC_286_TSS_B:
	case DESC_286_CALL_GATE:

	case DESC_386_TSS_A:		case DESC_386_TSS_B:
	case DESC_386_CALL_GATE:


	case DESC_DATA_EU_RO_NA:	case DESC_DATA_EU_RO_A:
	case DESC_DATA_EU_RW_NA:	case DESC_DATA_EU_RW_A:
	case DESC_DATA_ED_RO_NA:	case DESC_DATA_ED_RO_A:
	case DESC_DATA_ED_RW_NA:	case DESC_DATA_ED_RW_A:
	case DESC_CODE_N_NC_A:		case DESC_CODE_N_NC_NA:
	case DESC_CODE_R_NC_A:		case DESC_CODE_R_NC_NA:
		if (desc.DPL()<cpu.cpl || desc.DPL() < rpl) {
			SETFLAGBIT(ZF,false);
			return;
		}
		break;
	default:
		SETFLAGBIT(ZF,false);
		return;
	}
	/* Valid descriptor */
	ar=desc.saved.fill[1] & 0x00ffff00;
	SETFLAGBIT(ZF,true);
}

void CPU_LSL(Bitu selector,Bitu & limit) {
	FillFlags();
	if (selector == 0) {
		SETFLAGBIT(ZF,false);
		return;
	}
	Descriptor desc;Bitu rpl=selector & 3;
	if (!cpu.gdt.GetDescriptor(selector,desc)){
		SETFLAGBIT(ZF,false);
		return;
	}
	switch (desc.Type()){
	case DESC_CODE_N_C_A:	case DESC_CODE_N_C_NA:
	case DESC_CODE_R_C_A:	case DESC_CODE_R_C_NA:
		break;

	case DESC_LDT:
	case DESC_286_TSS_A:
	case DESC_286_TSS_B:

	case DESC_386_TSS_A:
	case DESC_386_TSS_B:

	case DESC_DATA_EU_RO_NA:	case DESC_DATA_EU_RO_A:
	case DESC_DATA_EU_RW_NA:	case DESC_DATA_EU_RW_A:
	case DESC_DATA_ED_RO_NA:	case DESC_DATA_ED_RO_A:
	case DESC_DATA_ED_RW_NA:	case DESC_DATA_ED_RW_A:

	case DESC_CODE_N_NC_A:		case DESC_CODE_N_NC_NA:
	case DESC_CODE_R_NC_A:		case DESC_CODE_R_NC_NA:
		if (desc.DPL()<cpu.cpl || desc.DPL() < rpl) {
			SETFLAGBIT(ZF,false);
			return;
		}
		break;
	default:
		SETFLAGBIT(ZF,false);
		return;
	}
	limit=desc.GetLimit();
	SETFLAGBIT(ZF,true);
}

void CPU_VERR(Bitu selector) {
	FillFlags();
	if (selector == 0) {
		SETFLAGBIT(ZF,false);
		return;
	}
	Descriptor desc;Bitu rpl=selector & 3;
	if (!cpu.gdt.GetDescriptor(selector,desc)){
		SETFLAGBIT(ZF,false);
		return;
	}
	switch (desc.Type()){
	case DESC_CODE_R_C_A:		case DESC_CODE_R_C_NA:
		//Conforming readable code segments can be always read
		break;
	case DESC_DATA_EU_RO_NA:	case DESC_DATA_EU_RO_A:
	case DESC_DATA_EU_RW_NA:	case DESC_DATA_EU_RW_A:
	case DESC_DATA_ED_RO_NA:	case DESC_DATA_ED_RO_A:
	case DESC_DATA_ED_RW_NA:	case DESC_DATA_ED_RW_A:

	case DESC_CODE_R_NC_A:		case DESC_CODE_R_NC_NA:
		if (desc.DPL()<cpu.cpl || desc.DPL() < rpl) {
			SETFLAGBIT(ZF,false);
			return;
		}
		break;
	default:
		SETFLAGBIT(ZF,false);
		return;
	}
	SETFLAGBIT(ZF,true);
}

void CPU_VERW(Bitu selector) {
	FillFlags();
	if (selector == 0) {
		SETFLAGBIT(ZF,false);
		return;
	}
	Descriptor desc;Bitu rpl=selector & 3;
	if (!cpu.gdt.GetDescriptor(selector,desc)){
		SETFLAGBIT(ZF,false);
		return;
	}
	switch (desc.Type()){
	case DESC_DATA_EU_RW_NA:	case DESC_DATA_EU_RW_A:
	case DESC_DATA_ED_RW_NA:	case DESC_DATA_ED_RW_A:
		if (desc.DPL()<cpu.cpl || desc.DPL() < rpl) {
			SETFLAGBIT(ZF,false);
			return;
		}
		break;
	default:
		SETFLAGBIT(ZF,false);
		return;
	}
	SETFLAGBIT(ZF,true);
}

bool CPU_SetSegGeneral(SegNames seg,Bitu value) {
	value &= 0xffff;
	if (!cpu.pmode || (reg_flags & FLAG_VM)) {
		Segs.val[seg]=value;
		Segs.phys[seg]=value << 4;
		if (seg==ss) {
			cpu.stack.big=false;
			cpu.stack.mask=0xffff;
			cpu.stack.notmask=0xffff0000;
		}
		return false;
	} else {
		if (seg==ss) {
			// Stack needs to be non-zero
			if ((value & 0xfffc)==0) {
				E_Exit("CPU_SetSegGeneral: Stack segment zero");
//				return CPU_PrepareException(EXCEPTION_GP,0);
			}
			Descriptor desc;
			if (!cpu.gdt.GetDescriptor(value,desc)) {
				E_Exit("CPU_SetSegGeneral: Stack segment beyond limits");
//				return CPU_PrepareException(EXCEPTION_GP,value & 0xfffc);
			}
			if (((value & 3)!=cpu.cpl) || (desc.DPL()!=cpu.cpl)) {
				E_Exit("CPU_SetSegGeneral: Stack segment with invalid privileges");
//				return CPU_PrepareException(EXCEPTION_GP,value & 0xfffc);
			}

			switch (desc.Type()) {
			case DESC_DATA_EU_RW_NA:		case DESC_DATA_EU_RW_A:
			case DESC_DATA_ED_RW_NA:		case DESC_DATA_ED_RW_A:
				break;
			default:
				//Earth Siege 1
				return CPU_PrepareException(EXCEPTION_GP,value & 0xfffc);
			}

			if (!desc.saved.seg.p) {
//				E_Exit("CPU_SetSegGeneral: Stack segment not present");	// or #SS(sel)
				return CPU_PrepareException(EXCEPTION_SS,value & 0xfffc);
			}

			Segs.val[seg]=value;
			Segs.phys[seg]=desc.GetBase();
			if (desc.Big()) {
				cpu.stack.big=true;
				cpu.stack.mask=0xffffffff;
				cpu.stack.notmask=0;
			} else {
				cpu.stack.big=false;
				cpu.stack.mask=0xffff;
				cpu.stack.notmask=0xffff0000;
			}
		} else {
			if ((value & 0xfffc)==0) {
				Segs.val[seg]=value;
				Segs.phys[seg]=0;	// ??
				return false;
			}
			Descriptor desc;
			if (!cpu.gdt.GetDescriptor(value,desc)) {
				return CPU_PrepareException(EXCEPTION_GP,value & 0xfffc);
			}
			switch (desc.Type()) {
			case DESC_DATA_EU_RO_NA:		case DESC_DATA_EU_RO_A:
			case DESC_DATA_EU_RW_NA:		case DESC_DATA_EU_RW_A:
			case DESC_DATA_ED_RO_NA:		case DESC_DATA_ED_RO_A:
			case DESC_DATA_ED_RW_NA:		case DESC_DATA_ED_RW_A:
			case DESC_CODE_R_NC_A:			case DESC_CODE_R_NC_NA:
				if (((value & 3)>desc.DPL()) || (cpu.cpl>desc.DPL())) {
					// extreme pinball
					return CPU_PrepareException(EXCEPTION_GP,value & 0xfffc);
				}
				break;
			case DESC_CODE_R_C_A:			case DESC_CODE_R_C_NA:
				break;
			default:
				// gabriel knight
				return CPU_PrepareException(EXCEPTION_GP,value & 0xfffc);

			}
			if (!desc.saved.seg.p) {
				// win
				return CPU_PrepareException(EXCEPTION_NP,value & 0xfffc);
			}

			Segs.val[seg]=value;
			Segs.phys[seg]=desc.GetBase();
		}

		return false;
	}
}

bool CPU_PopSeg(SegNames seg,bool use32) {
	Bitu val=mem_readw(SegPhys(ss) + (reg_esp & cpu.stack.mask));
	if (CPU_SetSegGeneral(seg,val)) return true;
	Bitu addsp=use32?0x04:0x02;
	reg_esp=(reg_esp&cpu.stack.notmask)|((reg_esp+addsp)&cpu.stack.mask);
	return false;
}

bool CPU_CPUID(void) {
	if (CPU_ArchitectureType<CPU_ARCHTYPE_486NEWSLOW) return false;
	switch (reg_eax) {
	case 0:	/* Vendor ID String and maximum level? */
		reg_eax=1;  /* Maximum level */
		reg_ebx='G' | ('e' << 8) | ('n' << 16) | ('u'<< 24);
		reg_edx='i' | ('n' << 8) | ('e' << 16) | ('I'<< 24);
		reg_ecx='n' | ('t' << 8) | ('e' << 16) | ('l'<< 24);
		break;
	case 1:	/* get processor type/family/model/stepping and feature flags */
		if ((CPU_ArchitectureType==CPU_ARCHTYPE_486NEWSLOW) ||
			(CPU_ArchitectureType==CPU_ARCHTYPE_MIXED)) {
			reg_eax=0x402;		/* intel 486dx */
			reg_ebx=0;			/* Not Supported */
			reg_ecx=0;			/* No features */
			reg_edx=0x00000001;	/* FPU */
		} else if (CPU_ArchitectureType==CPU_ARCHTYPE_PENTIUMSLOW) {
			reg_eax=0x513;		/* intel pentium */
			reg_ebx=0;			/* Not Supported */
			reg_ecx=0;			/* No features */
			reg_edx=0x00000011;	/* FPU+TimeStamp/RDTSC */
		} else {
			return false;
		}
		break;
	default:
		LOG(LOG_CPU,LOG_ERROR)("Unhandled CPUID Function %x",reg_eax);
		reg_eax=0;
		reg_ebx=0;
		reg_ecx=0;
		reg_edx=0;
		break;
	}
	return true;
}

static Bits HLT_Decode(void) {
	/* Once an interrupt occurs, it should change cpu core */
	if (reg_eip!=cpu.hlt.eip || SegValue(cs) != cpu.hlt.cs) {
		cpudecoder=cpu.hlt.old_decoder;
	} else {
		CPU_Cycles=0;
	}
	return 0;
}

void CPU_HLT(Bitu oldeip) {
	reg_eip=oldeip;
	CPU_Cycles=0;
	cpu.hlt.cs=SegValue(cs);
	cpu.hlt.eip=reg_eip;
	cpu.hlt.old_decoder=cpudecoder;
	cpudecoder=&HLT_Decode;
}

void CPU_ENTER(bool use32,Bitu bytes,Bitu level) {
	level&=0x1f;
	Bitu sp_index=reg_esp&cpu.stack.mask;
	Bitu bp_index=reg_ebp&cpu.stack.mask;
	if (!use32) {
		sp_index-=2;
		mem_writew(SegPhys(ss)+sp_index,reg_bp);
		reg_bp=(Bit16u)(reg_esp-2);
		if (level) {
			for (Bitu i=1;i<level;i++) {
				sp_index-=2;bp_index-=2;
				mem_writew(SegPhys(ss)+sp_index,mem_readw(SegPhys(ss)+bp_index));
			}
			sp_index-=2;
			mem_writew(SegPhys(ss)+sp_index,reg_bp);
		}
	} else {
		sp_index-=4;
        mem_writed(SegPhys(ss)+sp_index,reg_ebp);
		reg_ebp=(reg_esp-4);
		if (level) {
			for (Bitu i=1;i<level;i++) {
				sp_index-=4;bp_index-=4;
				mem_writed(SegPhys(ss)+sp_index,mem_readd(SegPhys(ss)+bp_index));
			}
			sp_index-=4;
			mem_writed(SegPhys(ss)+sp_index,reg_ebp);
		}
	}
	sp_index-=bytes;
	reg_esp=(reg_esp&cpu.stack.notmask)|((sp_index)&cpu.stack.mask);
}

static void CPU_CycleIncrease(bool pressed) {
	if (!pressed) return;
	if (CPU_CycleAutoAdjust) {
		CPU_CyclePercUsed+=5;
		if (CPU_CyclePercUsed>105) CPU_CyclePercUsed=105;
		LOG_MSG("CPU speed: max %d percent.",CPU_CyclePercUsed);
		GFX_SetTitle(CPU_CyclePercUsed,-1,false);
	} else {
		Bit32s old_cycles=CPU_CycleMax;
		if (CPU_CycleUp < 100) {
			CPU_CycleMax = (Bit32s)(CPU_CycleMax * (1 + (float)CPU_CycleUp / 100.0));
		} else {
			CPU_CycleMax = (Bit32s)(CPU_CycleMax + CPU_CycleUp);
		}

		CPU_CycleLeft=0;CPU_Cycles=0;
		if (CPU_CycleMax==old_cycles) CPU_CycleMax++;
		if(CPU_CycleMax > 15000 )
			LOG_MSG("CPU speed: fixed %d cycles. If you need more than 20000, try core=dynamic in DOSBox's options.",CPU_CycleMax);
		else
			LOG_MSG("CPU speed: fixed %d cycles.",CPU_CycleMax);
		GFX_SetTitle(CPU_CycleMax,-1,false);
	}
}

static void CPU_CycleDecrease(bool pressed) {
	if (!pressed) return;
	if (CPU_CycleAutoAdjust) {
		CPU_CyclePercUsed-=5;
		if (CPU_CyclePercUsed<=0) CPU_CyclePercUsed=1;
		if(CPU_CyclePercUsed <=70)
			LOG_MSG("CPU speed: max %d percent. If the game runs too fast, try a fixed cycles amount in DOSBox's options.",CPU_CyclePercUsed);
		else
			LOG_MSG("CPU speed: max %d percent.",CPU_CyclePercUsed);
		GFX_SetTitle(CPU_CyclePercUsed,-1,false);
	} else {
		if (CPU_CycleDown < 100) {
			CPU_CycleMax = (Bit32s)(CPU_CycleMax / (1 + (float)CPU_CycleDown / 100.0));
		} else {
			CPU_CycleMax = (Bit32s)(CPU_CycleMax - CPU_CycleDown);
		}
		CPU_CycleLeft=0;CPU_Cycles=0;
		if (CPU_CycleMax <= 0) CPU_CycleMax=1;
		LOG_MSG("CPU speed: fixed %d cycles.",CPU_CycleMax);
		GFX_SetTitle(CPU_CycleMax,-1,false);
	}
}

void CPU_Enable_SkipAutoAdjust(void) {
	if (CPU_CycleAutoAdjust) {
		CPU_CycleMax /= 2;
		if (CPU_CycleMax < CPU_CYCLES_LOWER_LIMIT)
			CPU_CycleMax = CPU_CYCLES_LOWER_LIMIT;
	}
	CPU_SkipCycleAutoAdjust=true;
}

void CPU_Disable_SkipAutoAdjust(void) {
	CPU_SkipCycleAutoAdjust=false;
}


extern Bit32s ticksDone;
extern Bit32u ticksScheduled;

void CPU_Reset_AutoAdjust(void) {
	CPU_IODelayRemoved = 0;
	ticksDone = 0;
	ticksScheduled = 0;
}

class CPU: public Module_base {
private:
	static bool inited;
public:
	CPU(Section* configuration):Module_base(configuration) {
		if(inited) {
			Change_Config(configuration);
			return;
		}
//		Section_prop * section=static_cast<Section_prop *>(configuration);
		inited=true;
		reg_eax=0;
		reg_ebx=0;
		reg_ecx=0;
		reg_edx=0;
		reg_edi=0;
		reg_esi=0;
		reg_ebp=0;
		reg_esp=0;

		SegSet16(cs,0);
		SegSet16(ds,0);
		SegSet16(es,0);
		SegSet16(fs,0);
		SegSet16(gs,0);
		SegSet16(ss,0);

		CPU_SetFlags(FLAG_IF,FMASK_ALL);		//Enable interrupts
		cpu.cr0=0xffffffff;
		CPU_SET_CRX(0,0);						//Initialize
		cpu.code.big=false;
		cpu.stack.mask=0xffff;
		cpu.stack.notmask=0xffff0000;
		cpu.stack.big=false;
		cpu.trap_skip=false;
		cpu.idt.SetBase(0);
		cpu.idt.SetLimit(1023);

		for (Bitu i=0; i<7; i++) {
			cpu.drx[i]=0;
			cpu.trx[i]=0;
		}
		if (CPU_ArchitectureType==CPU_ARCHTYPE_PENTIUMSLOW) {
			cpu.drx[6]=0xffff0ff0;
		} else {
			cpu.drx[6]=0xffff1ff0;
		}
		cpu.drx[7]=0x00000400;

		/* Init the cpu cores */
		CPU_Core_Normal_Init();
		CPU_Core_Simple_Init();
		CPU_Core_Full_Init();
#if (C_DYNAMIC_X86)
		CPU_Core_Dyn_X86_Init();
#elif (C_DYNREC)
		CPU_Core_Dynrec_Init();
#endif
		MAPPER_AddHandler(CPU_CycleDecrease,MK_f11,MMOD1,"cycledown","Dec Cycles");
		MAPPER_AddHandler(CPU_CycleIncrease,MK_f12,MMOD1,"cycleup"  ,"Inc Cycles");
		Change_Config(configuration);
		CPU_JMP(false,0,0,0);					//Setup the first cpu core
	}
	bool Change_Config(Section* newconfig){
		Section_prop * section=static_cast<Section_prop *>(newconfig);
		CPU_AutoDetermineMode=CPU_AUTODETERMINE_NONE;
		//CPU_CycleLeft=0;//needed ?
		CPU_Cycles=0;
		CPU_SkipCycleAutoAdjust=false;

		Prop_multival* p = section->Get_multival("cycles");
		std::string type = p->GetSection()->Get_string("type");
		std::string str ;
		CommandLine cmd(0,p->GetSection()->Get_string("parameters"));
		if (type=="max") {
			CPU_CycleMax=0;
			CPU_CyclePercUsed=100;
			CPU_CycleAutoAdjust=true;
			CPU_CycleLimit=-1;
			for (Bitu cmdnum=1; cmdnum<=cmd.GetCount(); cmdnum++) {
				if (cmd.FindCommand(cmdnum,str)) {
					if (str.find('%')==str.length()-1) {
						str.erase(str.find('%'));
						int percval=0;
						std::istringstream stream(str);
						stream >> percval;
						if ((percval>0) && (percval<=105)) CPU_CyclePercUsed=(Bit32s)percval;
					} else if (str=="limit") {
						cmdnum++;
						if (cmd.FindCommand(cmdnum,str)) {
							int cyclimit=0;
							std::istringstream stream(str);
							stream >> cyclimit;
							if (cyclimit>0) CPU_CycleLimit=cyclimit;
						}
					}
				}
			}
		} else {
			if (type=="auto") {
				CPU_AutoDetermineMode|=CPU_AUTODETERMINE_CYCLES;
				CPU_CycleMax=3000;
				CPU_OldCycleMax=3000;
				CPU_CyclePercUsed=100;
				for (Bitu cmdnum=0; cmdnum<=cmd.GetCount(); cmdnum++) {
					if (cmd.FindCommand(cmdnum,str)) {
						if (str.find('%')==str.length()-1) {
							str.erase(str.find('%'));
							int percval=0;
							std::istringstream stream(str);
							stream >> percval;
							if ((percval>0) && (percval<=105)) CPU_CyclePercUsed=(Bit32s)percval;
						} else if (str=="limit") {
							cmdnum++;
							if (cmd.FindCommand(cmdnum,str)) {
								int cyclimit=0;
								std::istringstream stream(str);
								stream >> cyclimit;
								if (cyclimit>0) CPU_CycleLimit=cyclimit;
							}
						} else {
							int rmdval=0;
							std::istringstream stream(str);
							stream >> rmdval;
							if (rmdval>0) {
								CPU_CycleMax=(Bit32s)rmdval;
								CPU_OldCycleMax=(Bit32s)rmdval;
							}
						}
					}
				}
			} else if(type =="fixed") {
				cmd.FindCommand(1,str);
				int rmdval=0;
				std::istringstream stream(str);
				stream >> rmdval;
				CPU_CycleMax=(Bit32s)rmdval;
			} else {
				std::istringstream stream(type);
				int rmdval=0;
				stream >> rmdval;
				if(rmdval) CPU_CycleMax=(Bit32s)rmdval;
			}
			CPU_CycleAutoAdjust=false;
		}

		CPU_CycleUp=section->Get_int("cycleup");
		CPU_CycleDown=section->Get_int("cycledown");
		std::string core(section->Get_string("core"));
		cpudecoder=&CPU_Core_Normal_Run;
		if (core == "normal") {
			cpudecoder=&CPU_Core_Normal_Run;
		} else if (core =="simple") {
			cpudecoder=&CPU_Core_Simple_Run;
		} else if (core == "full") {
			cpudecoder=&CPU_Core_Full_Run;
		} else if (core == "auto") {
			cpudecoder=&CPU_Core_Normal_Run;
#if (C_DYNAMIC_X86)
			CPU_AutoDetermineMode|=CPU_AUTODETERMINE_CORE;
		}
		else if (core == "dynamic") {
			cpudecoder=&CPU_Core_Dyn_X86_Run;
			CPU_Core_Dyn_X86_SetFPUMode(true);
		} else if (core == "dynamic_nodhfpu") {
			cpudecoder=&CPU_Core_Dyn_X86_Run;
			CPU_Core_Dyn_X86_SetFPUMode(false);
#elif (C_DYNREC)
			CPU_AutoDetermineMode|=CPU_AUTODETERMINE_CORE;
		}
		else if (core == "dynamic") {
			cpudecoder=&CPU_Core_Dynrec_Run;
#else

#endif
		}

#if (C_DYNAMIC_X86)
		CPU_Core_Dyn_X86_Cache_Init((core == "dynamic") || (core == "dynamic_nodhfpu"));
#elif (C_DYNREC)
		CPU_Core_Dynrec_Cache_Init( core == "dynamic" );
#endif

		CPU_ArchitectureType = CPU_ARCHTYPE_MIXED;
		std::string cputype(section->Get_string("cputype"));
		if (cputype == "auto") {
			CPU_ArchitectureType = CPU_ARCHTYPE_MIXED;
		} else if (cputype == "386") {
			CPU_ArchitectureType = CPU_ARCHTYPE_386FAST;
		} else if (cputype == "386_prefetch") {
			CPU_ArchitectureType = CPU_ARCHTYPE_386FAST;
			if (core == "normal") {
				cpudecoder=&CPU_Core_Prefetch_Run;
				CPU_PrefetchQueueSize = 16;
			} else if (core == "auto") {
				cpudecoder=&CPU_Core_Prefetch_Run;
				CPU_PrefetchQueueSize = 16;
				CPU_AutoDetermineMode&=(~CPU_AUTODETERMINE_CORE);
			} else {
				E_Exit("prefetch queue emulation requires the normal core setting.");
			}
		} else if (cputype == "386_slow") {
			CPU_ArchitectureType = CPU_ARCHTYPE_386SLOW;
		} else if (cputype == "486_slow") {
			CPU_ArchitectureType = CPU_ARCHTYPE_486NEWSLOW;
		} else if (cputype == "486_prefetch") {
			CPU_ArchitectureType = CPU_ARCHTYPE_486NEWSLOW;
			if (core == "normal") {
				cpudecoder=&CPU_Core_Prefetch_Run;
				CPU_PrefetchQueueSize = 32;
			} else if (core == "auto") {
				cpudecoder=&CPU_Core_Prefetch_Run;
				CPU_PrefetchQueueSize = 32;
				CPU_AutoDetermineMode&=(~CPU_AUTODETERMINE_CORE);
			} else {
				E_Exit("prefetch queue emulation requires the normal core setting.");
			}
		} else if (cputype == "pentium_slow") {
			CPU_ArchitectureType = CPU_ARCHTYPE_PENTIUMSLOW;
		}

		if (CPU_ArchitectureType>=CPU_ARCHTYPE_486NEWSLOW) CPU_flag_id_toggle=FLAG_ID;
		else CPU_flag_id_toggle=0;


		if(CPU_CycleMax <= 0) CPU_CycleMax = 3000;
		if(CPU_CycleUp <= 0)   CPU_CycleUp = 500;
		if(CPU_CycleDown <= 0) CPU_CycleDown = 20;
		if (CPU_CycleAutoAdjust) GFX_SetTitle(CPU_CyclePercUsed,-1,false);
		else GFX_SetTitle(CPU_CycleMax,-1,false);
		return true;
	}
	~CPU(){ /* empty */};
};

static CPU * test;

void CPU_ShutDown(Section* sec) {
#if (C_DYNAMIC_X86)
	CPU_Core_Dyn_X86_Cache_Close();
#elif (C_DYNREC)
	CPU_Core_Dynrec_Cache_Close();
#endif
	delete test;
}

void CPU_Init(Section* sec) {
	test = new CPU(sec);
	sec->AddDestroyFunction(&CPU_ShutDown,true);
}
//initialize static members
bool CPU::inited=false;
