use std::{mem, ptr};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

use super::avl;
use super::block_ptr::BlockPtr;
use super::dvaddr::DVAddr;
use super::from_bytes::FromBytes;
use super::lzjb;
use super::uberblock::Uberblock;
use super::zfs;

pub struct Reader {
    pub disk: File,
}

impl Reader {
    // TODO: Error handling
    pub fn read(&mut self, start: usize, length: usize) -> Vec<u8> {
        let mut ret: Vec<u8> = vec![0; length*512];

        self.disk.seek(SeekFrom::Start(start as u64 * 512));
        self.disk.read(&mut ret);

        return ret;
    }

    pub fn write(&mut self, block: usize, data: &[u8; 512]) {
        self.disk.seek(SeekFrom::Start(block as u64 * 512));
        self.disk.write(data);
    }

    pub fn read_dva(&mut self, dva: &DVAddr) -> Vec<u8> {
        self.read(dva.sector() as usize, dva.asize() as usize)
    }

    pub fn read_block(&mut self, block_ptr: &BlockPtr) -> Result<Vec<u8>, String> {
        let data = self.read_dva(&block_ptr.dvas[0]);
        match block_ptr.compression() {
            2 => {
                // compression off
                Ok(data)
            }
            1 | 3 => {
                // lzjb compression
                let mut decompressed = vec![0; (block_ptr.lsize()*512) as usize];
                lzjb::decompress(&data, &mut decompressed);
                Ok(decompressed)
            }
            _ => Err("Error: not enough bytes".to_string()),
        }
    }

    pub fn read_type<T: FromBytes>(&mut self, block_ptr: &BlockPtr) -> Result<T, String> {
        let data = self.read_block(block_ptr);
        data.and_then(|data| T::from_bytes(&data[..]))
    }

    pub fn read_type_array<T: FromBytes>(&mut self,
                                         block_ptr: &BlockPtr,
                                         offset: usize)
                                         -> Result<T, String> {
        let data = self.read_block(block_ptr);
        data.and_then(|data| T::from_bytes(&data[offset * mem::size_of::<T>()..]))
    }

    pub fn uber(&mut self) -> Result<Uberblock, String> {
        let mut newest_uberblock: Option<Uberblock> = None;
        for i in 0..128 {
            if let Ok(uberblock) = Uberblock::from_bytes(&self.read(256 + i * 2, 2)) {
                let newest = match newest_uberblock {
                    Some(previous) => {
                        if uberblock.txg > previous.txg {
                            // Found a newer uberblock
                            true
                        } else {
                            false
                        }
                    }
                    // No uberblock yet, so first one we find is the newest
                    None => true,
                };

                if newest {
                    newest_uberblock = Some(uberblock);
                }
            }
        }

        match newest_uberblock {
            Some(uberblock) => Ok(uberblock),
            None => Err("Failed to find valid uberblock".to_string()),
        }
    }
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////

// pub struct Zio {
// Core information about this IO
// bookmark: ZBookmarkPhys,
// prop: ZioProp,
// zio_type: Type,
// child_type: Child,
// int     io_cmd,
// priority: Priority,
// reexecute: u8,
// state: [u8; NUM_WAIT_TYPES],
// txg: u64,
// spa_t       *io_spa,
// blkptr_t    *io_bp,
// blkptr_t    *io_bp_override,
// bp_copy: BlockPtr,
// list_t      io_parent_list,
// list_t      io_child_list,
// zio_link_t  *io_walk_link,
// zio_t       *logical,
// zio_transform_t *io_transform_stack,
//
// Callback info
// ready: DoneFunc,
// physdone: DoneFunc,
// done: DoneFunc,
// private: *void,
// prev_space_delta: i64, // DMU private
// bp_orig: BlockPtr,
//
// Data represented by this IO
// void *data,
// void *orig_data,
// size: u64,
// orig_size: u64,
//
// Stuff for the vdev stack
// vdev_t      *vd,
// void        *io_vsd,
// const zio_vsd_ops_t *io_vsd_ops,
//
// offset: u64,
// timestamp: hrtime_t, // submitted at
// delta: hrtime_t,     // vdev queue service delta
// delay: u64,          // vdev disk service delta (ticks)
// queue_node: avl::NodeId,
// offset_node: avl::NodeId,
//
// Internal pipeline state
// flags: Flag,
// stage: State,
// pipeline: State,
// orig_flags: ZioFlag,
// orig_stage: State,
// orig_pipeline: State,
// error: zfs::Error,
// child_error: [zfs::Error; NUM_CHILD_TYPES],
// children: [[u64; NUM_WAIT_TYPES]; NUM_CHILD_TYPES],
// child_count: u64,
// phys_children: u64,
// parent_count: u64,
// uint64_t    *stall,
// zio_t       *gang_leader,
// zio_gang_node_t *gang_tree,
// void        *executor,
// void        *waiter,
// kmutex_t lock,
// kcondvar_t cv,*/
//
// FMA state
// zio_cksum_report_t *io_cksum_report,
// uint64_t io_ena,
//
// Taskq dispatching state
// tqent: TaskqEnt,
// }
//
// impl Zio {
// pub fn root(spa: Option<&Spa>, zio_done_func_t *done, void *private, flags: Flag) -> Self {
// Self::null(None, spa, None, done, private, flags)
// }
//
// pub fn read(zio_t *pio, spa_t *spa, const blkptr_t *bp,
// void *data, uint64_t size, zio_done_func_t *done, void *private,
// zio_priority_t priority, enum zio_flag flags, const zbookmark_phys_t *zb) -> Self {
// zfs_blkptr_verify(spa, bp);
//
// let pipeline =
// if flags & ZIO_FLAG_DDT_CHILD {
// ZIO_DDT_CHILD_READ_PIPELINE
// } else { ZIO_READ_PIPELINE };
//
// Self::create(pio, spa, BP_PHYSICAL_BIRTH(bp), bp,
// data, size, done, private,
// Type::Read, priority, flags, None, 0, zb,
// State::Open, pipeline)
// }
//
// fn null(pio: Option<&Zio>, spa: Option<&Spa>, vd: Option<&vdev::Vdev>, zio_done_func_t *done,
// void *private, flags: Flag) -> Self {
// Self::create(pio, spa, 0, None, None, 0, done, private,
// Type::Null, Priority::Now, flags, vd, 0, None,
// State::Open, ZIO_INTERLOCK_PIPELINE)
// }
//
// fn create(zio_t *pio, spa_t *spa, txg: u64, bp: Option<&BlockPtr>,
// void *data, size: u64, zio_done_func_t *done, void *private,
// zio_type: Type, priority: Priority, flags: Flag,
// vd: Option<&vdev::Vdev>, offset: u64, zb: Option<&ZBookmarkPhys>,
// stage: State, pipeline: State)-> Self {
// assert!(size <= SPA_MAXBLOCKSIZE);
// assert!(util::p2_phase(size, SPA_MINBLOCKSIZE) == 0);
// assert!(util::p2_phase(offset, SPA_MINBLOCKSIZE) == 0);
//
// assert!(!vd || spa_config_held(spa, SCL_STATE_ALL, RW_READER));
// assert!(!bp || !(flags & ZIO_FLAG_CONFIG_WRITER));
// assert!(vd || stage == ZIO_STAGE_OPEN);
//
// zio = kmem_cache_alloc(zcache, KM_SLEEP);
// bzero(zio, sizeof (zt));
//
// mutex_init(&zio->lock, NULL, MUTEX_DEFAULT, NULL);
// cv_init(&zio->cv, NULL, CV_DEFAULT, NULL);
//
// list_create(&zio->parent_list, sizeof (zlink_t),
// offsetof(zlink_t, zl_parent_node));
// list_create(&zio->child_list, sizeof (zlink_t),
// offsetof(zlink_t, zl_child_node));
//
// let child_type =
// if vd.is_some() {
// Child::Vdev
// } else if flags & ZIO_FLAG_GANG_CHILD {
// Child::Gang
// } else if flags & ZIO_FLAG_DDT_CHILD {
// Child::Ddt
// } else {
// Child::Logical
// };
//
// if let Some(bp) = bp {
// zio.bp = (blkptr_t *)bp;
// zio.bp_copy = *bp;
// zio.bp_orig = *bp;
// if zio_type != Type::Write || child_type == Child::Ddt {
// zio.bp = &zio.bp_copy; // so caller can free
// }
// if child_type == Child::Logical {
// zio.logical = zio;
// }
// if child_type > Child::Gang && BP_IS_GANG(bp) {
// pipeline |= ZIO_GANG_STAGES;
// }
// }
//
// if zb != NULL {
// zio.bookmark = *zb;
// }
//
// if let Some(pio) = pio {
// if zio.logical == NULL {
// zio.logical = pio.logical;
// }
// if zio.child_type == Child::Gang {
// zio.gang_leader = pio.gang_leader;
// }
// Self::add_child(pio, zio);
// }
//
// taskq::taskq_init_ent(&zio->tqent);
//
// Zio {
// child_type: child_type,
// spa: spa,
// txg: txg,
// done: done,
// private: private,
// zio_type: zio_type,
// priority: priority,
// vd: vd,
// offset: offset,
//
// data: data,
// orig_data: data,
// size: size,
// orig_size: size,
//
// flags: flags,
// orig_flags: flags,
// stage: stage,
// orig_stage: stage,
// pipeline: pipeline,
// orig_pipeline: pipeline,
//
// state: [stage >= State::Ready,
// state >= State::Done],
// }
// }
//
// fn read_phys(zio_t *pio, vdev_t *vd, offset: u64, size: u64,
// void *data, int checksum, zio_done_func_t *done, void *private,
// priority: Priority, zio_flag flags, labels: bool) -> Zio {
// assert!(vd->vdev_children == 0);
// assert!(!labels || offset + size <= VDEV_LABEL_START_SIZE ||
// offset >= vd.vdev_psize - VDEV_LABEL_END_SIZE);
// assert!(offset + size <= vd.vdev_psize);
//
// let mut zio = Self::create(pio, vd.vdev_spa, 0, NULL, data, size, done, private,
// Type::Read, priority, flags | ZIO_FLAG_PHYSICAL, vd, offset,
// NULL, State::Open, ZIO_READ_PHYS_PIPELINE);
//
// zio.prop.checksum = checksum;
//
// zio
// }
//
// ==========================================================================
// Parent/Child relationships
// ==========================================================================
//
// fn add_child(parent: &mut Zio, child: &mut Zio) {
// zio_link_t *zl = kmem_cache_alloc(zio_link_cache, KM_SLEEP);
// int w;
//
// Logical I/Os can have logical, gang, or vdev children.
// Gang I/Os can have gang or vdev children.
// Vdev I/Os can only have vdev children.
// The following assert captures all of these constraints.
// assert!(cio->io_child_type <= pio->io_child_type);
//
// zl.parent = parent;
// zl.child = child;
//
// mutex_enter(&child.lock);
// mutex_enter(&parent.lock);
//
// assert!(parent.state[WaitType::Done] == 0);
//
// for w in 0..NUM_WAIT_TYPES {
// parent.children[child.child_type][w] += !child.state[w];
// }
//
// list_insert_head(&pio->io_child_list, zl);
// list_insert_head(&cio->io_parent_list, zl);
//
// parent.child_count += 1;
// child.parent_count += 1;
//
// mutex_exit(&pio->io_lock);
// mutex_exit(&cio->io_lock);
// }
//
// ==========================================================================
// Execute the IO pipeline
// ==========================================================================
//
// fn taskq_dispatch(&mut self, mut tq_type: TaskqType, cut_in_line: bool) {
// let spa = self.spa;
// let flags = if cut_in_line { TQ_FRONT } else { 0 };
//
// let zio_type =
// if self.flags & (FLAG_CONFIG_WRITER | FLAG_PROBE) != 0 {
// If we're a config writer or a probe, the normal issue and
// interrupt threads may all be blocked waiting for the config lock.
// In this case, select the otherwise-unused taskq for ZIO_TYPE_NULL.
// Type::Null
// } else if self.zio_type == Type::Write && self.vd.is_some() && self.vd.vdev_aux {
// A similar issue exists for the L2ARC write thread until L2ARC 2.0.
// Type::Null
// } else {
// self.zio_type
// };
//
// If this is a high priority IO, then use the high priority taskq if
// available.
// if self.priority == Priority::Now && spa->spa_zio_taskq[t][tq_type + 1].stqs_count != 0 {
// tq_type += 1;
// }
//
// assert!(tq_type < NUM_TASKQ_TYPES);
//
// NB: We are assuming that the zio can only be dispatched
// to a single taskq at a time. It would be a grievous error
// to dispatch the zio to another taskq at the same time.
// assert!(taskq_empty_ent(&zio.tqent));
// spa.taskq_dispatch_ent(zio_type, tq_type, Box::new(|| { self.execute() }), flags, &self.tqent);
// }
//
// fn taskq_member(&self, TaskqType q) -> bool {
// let spa = self.spa;
//
// for t in 0..NUM_ZIO_TYPES {
// let tqs = &spa.zio_taskq[t][q];
// for i in 0..tqs.count {
// if tqs.taskq[i].member(self.executor) {
// return true;
// }
// }
// }
//
// false
// }
//
// fn issue_async(&self) -> PipelineFlow {
// self.taskq_dispatch(TaskqType::Issue, false);
//
// PipelineFlow::Stop
// }
//
// fn interrupt(&self) {
// self.taskq_dispatch(TaskqType::Interrupt, false);
// }
//
// Execute the I/O pipeline until one of the following occurs:
// (1) the I/O completes; (2) the pipeline stalls waiting for
// dependent child I/Os; (3) the I/O issues, so we're waiting
// for an I/O completion interrupt; (4) the I/O is delegated by
// vdev-level caching or aggregation; (5) the I/O is deferred
// due to vdev-level queueing; (6) the I/O is handed off to
// another thread.  In all cases, the pipeline stops whenever
// there's no CPU work; it never burns a thread in cv_wait_io().
//
// There's no locking on io_stage because there's no legitimate way
// for multiple threads to be attempting to process the same I/O.
// fn execute(&mut self) {
// self.executor = curthread;
//
// while self.stage < State::Done {
// let mut stage = self.stage;
//
// assert!(!MUTEX_HELD(&self.io_lock));
// assert!(ISP2(stage));
// assert!(self.stall == NULL);
// while stage & self.pipeline == 0 {
// stage <<= 1;
// }
//
// assert!(stage <= State::Done);
//
// let cut =
// match stage {
// State::VdevIoStart => REQUEUE_IO_START_CUT_IN_LINE,
// _ => false,
// };
//
// If we are in interrupt context and this pipeline stage
// will grab a config lock that is held across IO,
// or may wait for an IO that needs an interrupt thread
// to complete, issue async to avoid deadlock.
//
// For VDEV_IO_START, we cut in line so that the io will
// be sent to disk promptly.
// if stage & BLOCKING_STAGES != 0 && self.vd.is_none() && self.taskq_member(TaskqType::Interrupt) {
// self.taskq_dispatch(TaskqType::Issue, cut);
// return;
// }
//
// If we executing in the context of the tx_sync_thread,
// or we are performing pool initialization outside of a
// zio_taskq[ZIO_TASKQ_ISSUE|ZIO_TASKQ_ISSUE_HIGH] context.
// Then issue the zio asynchronously to minimize stack usage
// for these deep call paths.
// let dp = self.spa.get_dsl_pool();
// if (dp && curthread == dp.tx.tx_sync_thread) ||
// (dp && dp.spa.is_initializing() && !self.taskq_member(TaskqType::Issue) &&
// !self.taskq_member(TaskqType::IssueHigh)) {
// self.taskq_dispatch(TaskqType::Issue, cut);
// return;
// }*/
//
// self.stage = stage;
// let rv = pipeline_stages[highbit64(stage) - 1](self);
//
// if rv == PipelineFlow::Stop {
// return;
// }
//
// assert!(rv == PipelineFlow::Continue);
// }
// }
//
// pub fn wait(&self) -> zfs::Result<()> {
// assert!(self.stage == State::Open);
// assert!(self.executor == NULL);
//
// self.waiter = curthread;
//
// self.execute();
//
// mutex_enter(&self.lock);
// while self.executor != NULL {
// cv_wait_io(&self.cv, &self.lock);
// }
// mutex_exit(&self.lock);
//
// let error = self.error;
// self.destroy();
//
// Ok(())
// }
//
// fn no_wait(&mut self) {
// assert!(self.executor == NULL);
//
// if self.child_type == Child::Logical && self.unique_parent() == NULL {
// This is a logical async I/O with no parent to wait for it.
// We add it to the spa_async_root_zio "Godfather" I/O which
// will ensure they complete prior to unloading the pool.
// kpreempt_disable();
// let pio = self.spa.async_zio_root[CPU_SEQID];
// kpreempt_enable();
//
// Self::add_child(pio, self);
// }
//
// self.execute();
// }
//
// /////////////////////////////////////////////////////////////////////////////////////////////
// Pipeline stages
// /////////////////////////////////////////////////////////////////////////////////////////////
//
// fn read_bp_init(zio_t *zio) -> PipelineFlow {
// blkptr_t *bp = zio.bp;
//
// if (BP_GET_COMPRESS(bp) != ZIO_COMPRESS_OFF &&
// zio.child_type == Child::Logical &&
// !(zio->io_flags & ZIO_FLAG_RAW)) {
// uint64_t psize = BP_IS_EMBEDDED(bp) ? BPE_GET_PSIZE(bp) : BP_GET_PSIZE(bp);
// void *cbuf = zio_buf_alloc(psize);
//
// zio_push_transform(zio, cbuf, psize, psize, zio_decompress);
// }
//
// if BP_IS_EMBEDDED(bp) && BPE_GET_ETYPE(bp) == BP_EMBEDDED_TYPE_DATA {
// zio.pipeline = ZIO_INTERLOCK_PIPELINE;
// decode_embedded_bp_compressed(bp, zio->io_data);
// } else {
// ASSERT(!BP_IS_EMBEDDED(bp));
// }
//
// if !DMU_OT_IS_METADATA(BP_GET_TYPE(bp)) && BP_GET_LEVEL(bp) == 0 {
// zio.flags |= ZIO_FLAG_DONT_CACHE;
// }
//
// if BP_GET_TYPE(bp) == DMU_OT_DDT_ZAP {
// zio.flags |= ZIO_FLAG_DONT_CACHE;
// }
//
// if BP_GET_DEDUP(bp) && zio.child_type == Child::Logical {
// zio.pipeline = ZIO_DDT_READ_PIPELINE;
// }
//
// return PipelineFlow::Continue;
// }
//
// Issue an I/O to the underlying vdev. Typically the issue pipeline
// stops after this stage and will resume upon I/O completion.
// However, there are instances where the vdev layer may need to
// continue the pipeline when an I/O was not issued. Since the I/O
// that was sent to the vdev layer might be different than the one
// currently active in the pipeline (see vdev_queue_io()), we explicitly
// force the underlying vdev layers to call either zio_execute() or
// zio_interrupt() to ensure that the pipeline continues with the correct I/O.
// fn vdev_io_start(zio_t *zio) -> PipelineFlow {
// vdev_t *vd = zio.vd;
// spa_t *spa = zio.spa;
//
// assert!(zio.error == 0);
// assert!(zio.child_error[Child::Vdev] == 0);
//
// if vd == NULL {
// if zio.flags & ZIO_FLAG_CONFIG_WRITER == 0 {
// spa_config_enter(spa, SCL_ZIO, zio, RW_READER);
// }
//
// The mirror_ops handle multiple DVAs in a single BP.
// vdev_mirror_ops.vdev_op_start(zio);
// return PipelineFlow::Stop;
// }
//
// We keep track of time-sensitive I/Os so that the scan thread
// can quickly react to certain workloads.  In particular, we care
// about non-scrubbing, top-level reads and writes with the following
// characteristics:
//    - synchronous writes of user data to non-slog devices
//    - any reads of user data
// When these conditions are met, adjust the timestamp of spa_last_io
// which allows the scan thread to adjust its workload accordingly.
// if zio.flags & ZIO_FLAG_SCAN_THREAD == 0 && zio.bp != NULL && vd == vd.top_vdev &&
// !vd.is_log && zio.bookmark.objset != DMU_META_OBJSET && zio.txg != spa.syncing_txg() {
// let old = spa.spa_last_io;
// let new = ddi_get_lbolt64();
// if old != new {
// atomic_cas_64(&spa.spa_last_io, old, new);
// }
// }
//
// let align = 1 << vd.top_vdev.ashift;
//
// if zio.flags & ZIO_FLAG_PHYSICAL == 0 && util::p2_phase(zio.size, align) != 0 {
// Transform logical writes to be a full physical block size.
// let asize = util::p2_round_up(zio.size, align);
// char *abuf = zio_buf_alloc(asize);
// assert!(vd == vd.vdev_top);
// if (zio.zio_type == Type::Write) {
// bcopy(zio.data, abuf, zio.size);
// bzero(abuf + zio.size, asize - zio.size);
// }
// zio_push_transform(zio, abuf, asize, asize, zsubblock);
// }
//
// If this is not a physical io, make sure that it is properly aligned
// before proceeding.
// if zio.flags & ZIO_FLAG_PHYSICAL == 0 {
// assert!(util::p2_phase(zio.offset, align) == 0);
// assert!(util::p2_phase(zio.size, align) == 0);
// } else {
// For physical writes, we allow 512b aligned writes and assume
// the device will perform a read-modify-write as necessary.
// assert!(util::p2_phase(zio.offset, SPA_MINBLOCKSIZE) == 0);
// assert!(util::p2_phase(zio.size, SPA_MINBLOCKSIZE) == 0);
// }
//
// VERIFY(zio.zio_type != Type::Write || spa_writeable(spa));
//
// If this is a repair I/O, and there's no self-healing involved --
// that is, we're just resilvering what we expect to resilver --
// then don't do the I/O unless zio's txg is actually in vd's DTL.
// This prevents spurious resilvering with nested replication.
// For example, given a mirror of mirrors, (A+B)+(C+D), if only
// A is out of date, we'll read from C+D, then use the data to
// resilver A+B -- but we don't actually want to resilver B, just A.
// The top-level mirror has no way to know this, so instead we just
// discard unnecessary repairs as we work our way down the vdev tree.
// The same logic applies to any form of nested replication:
// ditto + mirror, RAID-Z + replacing, etc. This covers them all.
// if (zio.flags & ZIO_FLAG_IO_REPAIR != 0 &&
// zio.flags & ZIO_FLAG_SELF_HEAL == 0 &&
// zio.txg != 0 &&    /* not a delegated i/o */
// !vdev_dtl_contains(vd, DTL_PARTIAL, zio.txg, 1)) {
// assert!(zio.zio_type == Type::Write);
// zio_vdev_bypass(zio);
// return PipelineFlow::Continue;
// }
//
// if vd.ops.is_leaf() && (zio.zio_type == Type::Read || zio.zio_type == Type::Write) {
// if zio.zio_type == Type::Read && vdev_cache_read(zio) {
// return PipelineFlow::Continue;
// }
//
// if (zio = vdev_queue_io(zio)) == NULL {
// return PipelineFlow::Stop;
// }
//
// if !vdev_accessible(vd, zio) {
// zio.error = SET_ERROR(ENXIO);
// zio.interrupt();
// return PipelineFlow::Stop;
// }
// }
//
// (vd.ops.io_start)(zio);
// PipelineFlow::Stop
// }
//
// fn vdev_io_done(zio: &mut Zio) -> PipelineFlow {
// vdev_t *vd = zio.vd;
// vdev_ops_t *ops = vd ? vd->vdev_ops : &vdev_mirror_ops;
// let mut unexpected_error = false;
//
// if zio.wait_for_children(Child::Vdev, WaitType::Done) {
// return PipelineFlow::Stop;
// }
//
// assert!(zio.zio_type == Type::Read || zio.zio_type == Type::Write);
//
// if vd != NULL && vd.ops.is_leaf() {
// vdev_queue_io_done(zio);
//
// if zio.zio_type == Type::Write {
// vdev_cache_write(zio);
// }
//
// if zio_injection_enabled && zio.error == 0 {
// zio.error = zio_handle_device_injection(vd, zio, EIO);
// }
//
// if zio_injection_enabled && zio.error == 0 {
// zio.error = zio_handle_label_injection(zio, EIO);
// }*/
//
// if zio.error {
// if !vdev_accessible(vd, zio) {
// zio.error = SET_ERROR(ENXIO);
// } else {
// unexpected_error = true;
// }
// }
// }
//
// (ops.io_done)(zio);
//
// if unexpected_error {
// VERIFY(vdev_probe(vd, zio) == NULL);
// }
//
// PipelineFlow::Continue
// }
// }

/// /////////////////////////////////////////////////////////////////////////////////////////////////

// A bookmark is a four-tuple <objset, object, level, blkid> that uniquely
// identifies any block in the pool.  By convention, the meta-objset (MOS)
// is objset 0, and the meta-dnode is object 0.  This covers all blocks
// except root blocks and ZIL blocks, which are defined as follows:
//
// Root blocks (objset_phys_t) are object 0, level -1:  <objset, 0, -1, 0>.
// ZIL blocks are bookmarked <objset, 0, -2, blkid == ZIL sequence number>.
// dmu_sync()ed ZIL data blocks are bookmarked <objset, object, -2, blkid>.
//
// Note: this structure is called a bookmark because its original purpose
// was to remember where to resume a pool-wide traverse.
//
// Note: this structure is passed between userland and the kernel, and is
// stored on disk (by virtue of being incorporated into other on-disk
// structures, e.g. dsl_scan_phys_t).
//
struct ZbookmarkPhys {
    objset: u64,
    object: u64,
    level: i64,
    blkid: u64,
}

const REQUEUE_IO_START_CUT_IN_LINE: bool = true;
pub const NUM_CHILD_TYPES: usize = 4;
pub const NUM_WAIT_TYPES: usize = 2;
pub const NUM_TYPES: usize = 6;
pub const NUM_TASKQ_TYPES: usize = 4;

// Default Linux timeout for a sd device.
// const ZIO_DELAY_MAX = (30 * MILLISEC);

// const ZIO_FAILURE_MODE_WAIT = 0;
// const ZIO_FAILURE_MODE_CONTINUE = 1;
// const ZIO_FAILURE_MODE_PANIC = 2;

// pub enum TaskqType {
// Issue = 0,
// IssueHigh,
// Interrupt,
// InterruptHigh,
// }
//
// #[derive(Copy, Clone, PartialEq)]
// enum Priority {
// SyncRead,
// SyncWrite,  // ZIL
// AsyncRead,  // prefetch
// AsyncWrite, // spa_sync()
// Scrub,      // asynchronous scrub/resilver reads
// NumQueueable,
//
// Now         // non-queued io (e.g. free)
// }
//
// #[derive(Copy, Clone, PartialEq)]
// pub enum Type {
// Null = 0,
// Read,
// Write,
// Free,
// Claim,
// IoCtl,
// }
//
// const FLAG_AGG_INHERIT: u64 = Flag::CanFail - 1;
// const FLAG_DDT_INHERIT: u64 = Flag::IoRetry - 1;
// const FLAG_GANG_INHERIT: u64 = Flag::IoRetry - 1;
// const FLAG_VDEV_INHERIT: u64 = Flag::DontQueue - 1;
//
// const NUM_PIPE_STAGES: usize = 22;
//
// type PipeStageFn = fn(&mut Zio) -> zfs::Result<()>;
// static pipeline_stages: [Option<PipeStageFn>; NUM_PIPE_STAGES] =
// [None,
// Some(Zio::read_bp_init),
// None,//Some(Zio::free_bp_init),
// Some(Zio::issue_async),
// None,//Some(Zio::write_bp_init),
// None,//Some(Zio::checksum_generate),
// None,//Some(Zio::nop_write),
// None,//Some(Zio::ddt_read_start),
// None,//Some(Zio::ddt_read_done),
// None,//Some(Zio::ddt_write),
// None,//Some(Zio::ddt_free),
// None,//Some(Zio::gang_assemble),
// None,//Some(Zio::gang_issue),
// None,//Some(Zio::dva_allocate),
// None,//Some(Zio::dva_free),
// None,//Some(Zio::dva_claim),
// Some(Zio::ready),
// Some(Zio::vdev_io_start),
// Some(Zio::vdev_io_done),
// Some(Zio::vdev_io_assess),
// Some(Zio::checksum_verify),
// Some(Zio::done)];
//
// #[derive(Copy, Clone, PartialEq)]
// enum PipelineFlow {
// Continue = 0x100,
// Stop = 0x101,
// }
//
// #[derive(Copy, Clone, PartialEq)]
// enum Flag {
// Flags inherited by gang, ddt, and vdev children,
// and that must be equal for two zios to aggregate
// DontAggregate  = 1 << 0,
// IoRepair       = 1 << 1,
// SelfHeal       = 1 << 2,
// Resilver       = 1 << 3,
// Scrub          = 1 << 4,
// ScanThread     = 1 << 5,
// Physical       = 1 << 6,
//
// Flags inherited by ddt, gang, and vdev children.
// CanFail        = 1 << 7, // must be first for INHERIT
// Speculative    = 1 << 8,
// ConfigWriter   = 1 << 9,
// DontRetry      = 1 << 10,
// DontCache      = 1 << 11,
// NoData         = 1 << 12,
// InduceDamage   = 1 << 13,
//
// Flags inherited by vdev children.
// IoRetry        = 1 << 14,    /* must be first for INHERIT */
// Probe          = 1 << 15,
// TryHard        = 1 << 16,
// Optional       = 1 << 17,
//
// Flags not inherited by any children.
// DontQueue      = 1 << 18,    /* must be first for INHERIT */
// DontPropagate  = 1 << 19,
// IoBypass       = 1 << 20,
// IoRewrite      = 1 << 21,
// Raw            = 1 << 22,
// GangChild      = 1 << 23,
// DdtChild       = 1 << 24,
// GodFather      = 1 << 25,
// NopWrite       = 1 << 26,
// ReExecuted     = 1 << 27,
// Delegated      = 1 << 28,
// FastWrite      = 1 << 29,
// };
//
// #[derive(Copy, Clone, PartialEq)]
// enum Child {
// Vdev = 0,
// Gang,
// Ddt,
// Logical,
// };
//
// #[repr(u8)]
// enum WaitType {
// Ready = 0,
// Done,
// };
//
// zio pipeline stage definitions
// enum Stage {
// Open              = 1 << 0,  // RWFCI
//
// ReadBpInit        = 1 << 1,  // R----
// FreeBpInit        = 1 << 2,  // --F--
// IssueAsync        = 1 << 3,  // RWF--
// WriteBpInit       = 1 << 4,  // -W---
//
// ChecksumGenerate  = 1 << 5,  // -W---
//
// NopWrite          = 1 << 6,  // -W---
//
// DdtReadStart      = 1 << 7,  // R----
// DdtReadDone       = 1 << 8,  // R----
// DdtWrite          = 1 << 9,  // -W---
// DdtFree           = 1 << 10, // --F--
//
// GangAssemble      = 1 << 11, // RWFC-
// GangIssue         = 1 << 12, // RWFC-
//
// DvaAllocate       = 1 << 13, // -W---
// DvaFree           = 1 << 14, // --F--
// DvaClaim          = 1 << 15, // ---C-
//
// Ready             = 1 << 16, // RWFCI
//
// VdevIoStart       = 1 << 17, // RW--I
// VdevIoDone        = 1 << 18, // RW--I
// VdevIoAssess      = 1 << 19, // RW--I
//
// ChecksumVerify    = 1 << 20, // R----
//
// Done              = 1 << 21, // RWFCI
// };
//
// const INTERLOCK_STAGES = STAGE_READY | STAGE_DONE;
//
// const INTERLOCK_PIPELINE = INTERLOCK_STAGES
//
// const VDEV_IO_STAGES = STAGE_VDEV_IO_START |
// STAGE_VDEV_IO_DONE | STAGE_VDEV_IO_ASSESS;
//
// const VDEV_CHILD_PIPELINE = VDEV_IO_STAGES | STAGE_DONE;
//
// const READ_COMMON_STAGES = INTERLOCK_STAGES | VDEV_IO_STAGES | STAGE_CHECKSUM_VERIFY
//
// const READ_PHYS_PIPELINE = READ_COMMON_STAGES
//
// const READ_PIPELINE = READ_COMMON_STAGES | STAGE_READ_BP_INIT
//
// const DDT_CHILD_READ_PIPELINE = READ_COMMON_STAGES;
//
// const DDT_READ_PIPELINE = INTERLOCK_STAGES | STAGE_READ_BP_INIT | STAGE_DDT_READ_START | STAGE_DDT_READ_DONE;
//
// const WRITE_COMMON_STAGES = INTERLOCK_STAGES | VDEV_IO_STAGES | STAGE_ISSUE_ASYNC | STAGE_CHECKSUM_GENERATE;
//
// const WRITE_PHYS_PIPELINE = WRITE_COMMON_STAGES;
//
// const REWRITE_PIPELINE = WRITE_COMMON_STAGES | STAGE_WRITE_BP_INIT;
//
// const WRITE_PIPELINE = WRITE_COMMON_STAGES | STAGE_WRITE_BP_INIT | STAGE_DVA_ALLOCATE;
//
// const DDT_CHILD_WRITE_PIPELINE = INTERLOCK_STAGES | VDEV_IO_STAGES | STAGE_DVA_ALLOCATE;
//
// const DDT_WRITE_PIPELINE = INTERLOCK_STAGES | STAGE_ISSUE_ASYNC |
// STAGE_WRITE_BP_INIT | STAGE_CHECKSUM_GENERATE |
// STAGE_DDT_WRITE;
//
// const GANG_STAGES = STAGE_GANG_ASSEMBLE | STAGE_GANG_ISSUE;
//
// const FREE_PIPELINE = INTERLOCK_STAGES | STAGE_FREE_BP_INIT | STAGE_DVA_FREE;
//
// const DDT_FREE_PIPELINE = INTERLOCK_STAGES | STAGE_FREE_BP_INIT | STAGE_ISSUE_ASYNC | STAGE_DDT_FREE;
//
// const CLAIM_PIPELINE = INTERLOCK_STAGES | STAGE_DVA_CLAIM;
//
// const IOCTL_PIPELINE = INTERLOCK_STAGES | STAGE_VDEV_IO_START | STAGE_VDEV_IO_ASSESS;
//
// const BLOCKING_STAGES = STAGE_DVA_ALLOCATE | STAGE_DVA_CLAIM | STAGE_VDEV_IO_START;
//
