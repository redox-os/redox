use super::from_bytes::FromBytes;
use super::dvaddr::DVAddr;

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct BlockPtr {
    pub dvas: [DVAddr; 3],
    pub flags_size: u64,
    pub padding: [u64; 3],
    pub birth_txg: u64,
    pub fill_count: u64,
    pub checksum: [u64; 4],
}

impl BlockPtr {
    pub fn level(&self) -> u64 {
        (self.flags_size >> 56) & 0x7F
    }

    pub fn object_type(&self) -> u64 {
        (self.flags_size >> 48) & 0xFF
    }

    pub fn checksum(&self) -> u64 {
        (self.flags_size >> 40) & 0xFF
    }

    pub fn compression(&self) -> u64 {
        (self.flags_size >> 32) & 0xFF
    }

    pub fn lsize(&self) -> u64 {
        (self.flags_size & 0xFFFF) + 1
    }

    pub fn psize(&self) -> u64 {
        ((self.flags_size >> 16) & 0xFFFF) + 1
    }

#define	BP_GET_LSIZE(bp)	\
	(BP_IS_EMBEDDED(bp) ?	\
	(BPE_GET_ETYPE(bp) == BP_EMBEDDED_TYPE_DATA ? BPE_GET_LSIZE(bp) : 0): \
	BF64_GET_SB((bp)->blk_prop, 0, SPA_LSIZEBITS, SPA_MINBLOCKSHIFT, 1))
#define	BP_SET_LSIZE(bp, x)	do { \
	ASSERT(!BP_IS_EMBEDDED(bp)); \
	BF64_SET_SB((bp)->blk_prop, \
	    0, SPA_LSIZEBITS, SPA_MINBLOCKSHIFT, 1, x); \
_NOTE(CONSTCOND) } while (0)

#define	BP_GET_PSIZE(bp)	\
	(BP_IS_EMBEDDED(bp) ? 0 : \
	BF64_GET_SB((bp)->blk_prop, 16, SPA_PSIZEBITS, SPA_MINBLOCKSHIFT, 1))
#define	BP_SET_PSIZE(bp, x)	do { \
	ASSERT(!BP_IS_EMBEDDED(bp)); \
	BF64_SET_SB((bp)->blk_prop, \
	    16, SPA_PSIZEBITS, SPA_MINBLOCKSHIFT, 1, x); \
_NOTE(CONSTCOND) } while (0)

#define	BP_GET_COMPRESS(bp)		BF64_GET((bp)->blk_prop, 32, 7)
#define	BP_SET_COMPRESS(bp, x)		BF64_SET((bp)->blk_prop, 32, 7, x)

#define	BP_IS_EMBEDDED(bp)		BF64_GET((bp)->blk_prop, 39, 1)
#define	BP_SET_EMBEDDED(bp, x)		BF64_SET((bp)->blk_prop, 39, 1, x)

#define	BP_GET_CHECKSUM(bp)		\
	(BP_IS_EMBEDDED(bp) ? ZIO_CHECKSUM_OFF : \
	BF64_GET((bp)->blk_prop, 40, 8))
#define	BP_SET_CHECKSUM(bp, x)		do { \
	ASSERT(!BP_IS_EMBEDDED(bp)); \
	BF64_SET((bp)->blk_prop, 40, 8, x); \
_NOTE(CONSTCOND) } while (0)

#define	BP_GET_TYPE(bp)			BF64_GET((bp)->blk_prop, 48, 8)
#define	BP_SET_TYPE(bp, x)		BF64_SET((bp)->blk_prop, 48, 8, x)

#define	BP_GET_LEVEL(bp)		BF64_GET((bp)->blk_prop, 56, 5)
#define	BP_SET_LEVEL(bp, x)		BF64_SET((bp)->blk_prop, 56, 5, x)

#define	BP_GET_DEDUP(bp)		BF64_GET((bp)->blk_prop, 62, 1)
#define	BP_SET_DEDUP(bp, x)		BF64_SET((bp)->blk_prop, 62, 1, x)

#define	BP_GET_BYTEORDER(bp)		BF64_GET((bp)->blk_prop, 63, 1)
#define	BP_SET_BYTEORDER(bp, x)		BF64_SET((bp)->blk_prop, 63, 1, x)

#define	BP_PHYSICAL_BIRTH(bp)		\
	(BP_IS_EMBEDDED(bp) ? 0 : \
	(bp)->blk_phys_birth ? (bp)->blk_phys_birth : (bp)->blk_birth)

#define	BP_SET_BIRTH(bp, logical, physical)	\
{						\
	ASSERT(!BP_IS_EMBEDDED(bp));		\
	(bp)->blk_birth = (logical);		\
	(bp)->blk_phys_birth = ((logical) == (physical) ? 0 : (physical)); \
}

#define	BP_GET_FILL(bp) (BP_IS_EMBEDDED(bp) ? 1 : (bp)->blk_fill)

#define	BP_GET_ASIZE(bp)	\
	(BP_IS_EMBEDDED(bp) ? 0 : \
	DVA_GET_ASIZE(&(bp)->blk_dva[0]) + \
	DVA_GET_ASIZE(&(bp)->blk_dva[1]) + \
	DVA_GET_ASIZE(&(bp)->blk_dva[2]))

#define	BP_GET_UCSIZE(bp) \
	((BP_GET_LEVEL(bp) > 0 || DMU_OT_IS_METADATA(BP_GET_TYPE(bp))) ? \
	BP_GET_PSIZE(bp) : BP_GET_LSIZE(bp))

#define	BP_GET_NDVAS(bp)	\
	(BP_IS_EMBEDDED(bp) ? 0 : \
	!!DVA_GET_ASIZE(&(bp)->blk_dva[0]) + \
	!!DVA_GET_ASIZE(&(bp)->blk_dva[1]) + \
	!!DVA_GET_ASIZE(&(bp)->blk_dva[2]))

#define	BP_COUNT_GANG(bp)	\
	(BP_IS_EMBEDDED(bp) ? 0 : \
	(DVA_GET_GANG(&(bp)->blk_dva[0]) + \
	DVA_GET_GANG(&(bp)->blk_dva[1]) + \
	DVA_GET_GANG(&(bp)->blk_dva[2])))

#define	DVA_EQUAL(dva1, dva2)	\
	((dva1)->dva_word[1] == (dva2)->dva_word[1] && \
	(dva1)->dva_word[0] == (dva2)->dva_word[0])

#define	BP_EQUAL(bp1, bp2)	\
	(BP_PHYSICAL_BIRTH(bp1) == BP_PHYSICAL_BIRTH(bp2) &&	\
	(bp1)->blk_birth == (bp2)->blk_birth &&			\
	DVA_EQUAL(&(bp1)->blk_dva[0], &(bp2)->blk_dva[0]) &&	\
	DVA_EQUAL(&(bp1)->blk_dva[1], &(bp2)->blk_dva[1]) &&	\
	DVA_EQUAL(&(bp1)->blk_dva[2], &(bp2)->blk_dva[2]))
}

impl FromBytes for BlockPtr { }

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct Gang {
    pub bps: [BlockPtr; 3],
    pub padding: [u64; 14],
    pub magic: u64,
    pub checksum: u64,
}

impl Gang {
    pub fn magic() -> u64 {
        return 0x117a0cb17ada1002;
    }
}
