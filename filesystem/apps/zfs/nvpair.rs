enum DataType {
    Unknown = 0,
    Boolean,
    Byte,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Int64,
    Uint64,
    String,
    ByteArray,
    Int16Array,
    Uint16Array,
    Int32Array,
    Uint32Array,
    Int64Array,
    Uint64Array,
    StringArray,
    HrTime,
    NvList,
    NvListArray,
    BooleanValue,
    Int8,
    Uint8,
    BooleanArray,
    Int8Array,
    Uint8Array
}

struct NvPair {
    nvp_size:       i32, // size of this nvpair
    nvp_name_sz:    i16, // length of name string
    nvp_reserve:    i16, // not used
    nvp_value_elem: i32, // number of elements for array types
    nvp_type:       DataType, // type of value
    // name string
    // aligned ptr array for string arrays
    // aligned array of data for value
}

// nvlist header
struct NvList {
    nvl_version: i32
    nvl_nvflag:  u32 // persistent flags
    nvl_priv:    u64 // ptr to private data if not packed
    nvl_flag:    u32
    nvl_pad:     i32 // currently not used, for alignment
}

// nvp implementation version
const NV_VERSION: i32 = 0;

// nvlist pack encoding
const NV_ENCODE_NATIVE: u8 = 0;
const NV_ENCODE_XDR:    u8 = 1;

// nvlist persistent unique name flags, stored in nvl_nvflags
const NV_UNIQUE_NAME:      u32 = 0x1;
const NV_UNIQUE_NAME_TYPE: u32 = 0x2;

// nvlist lookup pairs related flags
const NV_FLAG_NOENTOK: isize = 0x1;

/* What to do about these macros?
// convenience macros
#define NV_ALIGN(x)     (((ulong_t)(x) + 7ul) & ~7ul)
#define NV_ALIGN4(x)        (((x) + 3) & ~3)

#define NVP_SIZE(nvp)       ((nvp)->nvp_size)
#define NVP_NAME(nvp)       ((char *)(nvp) + sizeof (nvpair_t))
#define NVP_TYPE(nvp)       ((nvp)->nvp_type)
#define NVP_NELEM(nvp)      ((nvp)->nvp_value_elem)
#define NVP_VALUE(nvp)      ((char *)(nvp) + NV_ALIGN(sizeof (nvpair_t) \
                + (nvp)->nvp_name_sz))

#define NVL_VERSION(nvl)    ((nvl)->nvl_version)
#define NVL_SIZE(nvl)       ((nvl)->nvl_size)
#define NVL_FLAG(nvl)       ((nvl)->nvl_flag)
*/

// NV allocator framework
struct NvAllocOps;

struct NvAlloc<> {
    nva_ops: &'static NvAllocOps,
    nva_arg: Any, // This was a void pointer type.
                  // Not sure if Any is the correct type.
}

/*
struct NvAllocOps {
    int (*nv_ao_init)(nv_alloc_t *, __va_list);
    void (*nv_ao_fini)(nv_alloc_t *);
    void *(*nv_ao_alloc)(nv_alloc_t *, size_t);
    void (*nv_ao_free)(nv_alloc_t *, void *, size_t);
    void (*nv_ao_reset)(nv_alloc_t *);
}
*/
