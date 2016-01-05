use super::zio;

// ZFS IO Scheduler
// ---------------
//
// ZFS issues IO operations to leaf vdevs to satisfy and complete zios.  The
// IO scheduler determines when and in what order those operations are
// issued.  The IO scheduler divides operations into five IO classes
// prioritized in the following order: sync read, sync write, async read,
// async write, and scrub/resilver.  Each queue defines the minimum and
// maximum number of concurrent operations that may be issued to the device.
// In addition, the device has an aggregate maximum. Note that the sum of the
// per-queue minimums must not exceed the aggregate maximum. If the
// sum of the per-queue maximums exceeds the aggregate maximum, then the
// number of active IOs may reach zfs_vdev_max_active, in which case no
// further IOs will be issued regardless of whether all per-queue
// minimums have been met.
//
// For many physical devices, throughput increases with the number of
// concurrent operations, but latency typically suffers. Further, physical
// devices typically have a limit at which more concurrent operations have no
// effect on throughput or can actually cause it to decrease.
//
// The scheduler selects the next operation to issue by first looking for an
// IO class whose minimum has not been satisfied. Once all are satisfied and
// the aggregate maximum has not been hit, the scheduler looks for classes
// whose maximum has not been satisfied. Iteration through the IO classes is
// done in the order specified above. No further operations are issued if the
// aggregate maximum number of concurrent operations has been hit or if there
// are no operations queued for an IO class that has not hit its maximum.
// Every time an IO is queued or an operation completes, the IO scheduler
// looks for new operations to issue.
//
// All IO classes have a fixed maximum number of outstanding operations
// except for the async write class. Asynchronous writes represent the data
// that is committed to stable storage during the syncing stage for
// transaction groups (see txg.c). Transaction groups enter the syncing state
// periodically so the number of queued async writes will quickly burst up and
// then bleed down to zero. Rather than servicing them as quickly as possible,
// the IO scheduler changes the maximum number of active async write IOs
// according to the amount of dirty data in the pool (see dsl_pool.c). Since
// both throughput and latency typically increase with the number of
// concurrent operations issued to physical devices, reducing the burstiness
// in the number of concurrent operations also stabilizes the response time of
// operations from other -- and in particular synchronous -- queues. In broad
// strokes, the IO scheduler will issue more concurrent operations from the
// async write queue as there's more dirty data in the pool.
//
// Async Writes
//
// The number of concurrent operations issued for the async write IO class
// follows a piece-wise linear function defined by a few adjustable points.
//
//        |                   o---------| <-- zfs_vdev_async_write_max_active
//   ^    |                  /^         |
//   |    |                 / |         |
// active |                /  |         |
//  IO   |               /   |         |
// count  |              /    |         |
//        |             /     |         |
//        |------------o      |         | <-- zfs_vdev_async_write_min_active
//       0|____________^______|_________|
//        0%           |      |       100% of zfs_dirty_data_max
//                     |      |
//                     |      `-- zfs_vdev_async_write_active_max_dirty_percent
//                     `--------- zfs_vdev_async_write_active_min_dirty_percent
//
// Until the amount of dirty data exceeds a minimum percentage of the dirty
// data allowed in the pool, the IO scheduler will limit the number of
// concurrent operations to the minimum. As that threshold is crossed, the
// number of concurrent operations issued increases linearly to the maximum at
// the specified maximum percentage of the dirty data allowed in the pool.
//
// Ideally, the amount of dirty data on a busy pool will stay in the sloped
// part of the function between zfs_vdev_async_write_active_min_dirty_percent
// and zfs_vdev_async_write_active_max_dirty_percent. If it exceeds the
// maximum percentage, this indicates that the rate of incoming data is
// greater than the rate that the backend storage can handle. In this case, we
// must further throttle incoming writes (see dmu_tx_delay() for details).

// the sum of each queue's max_active.  It must be at least the sum of each
// queue's min_active.
uint32_t zfs_vdev_max_active = 1000;

// Per-queue limits on the number of IOs active to each device.  If the
// number of active IOs is < zfs_vdev_max_active, then the min_active comes
// into play. We will send min_active from each queue, and then select from
// queues in the order defined by zio_priority_t.
//
// In general, smaller max_active's will lead to lower latency of synchronous
// operations.  Larger max_active's may lead to higher overall throughput,
// depending on underlying storage.
//
// The ratio of the queues' max_actives determines the balance of performance
// between reads, writes, and scrubs.  E.g., increasing
// zfs_vdev_scrub_max_active will cause the scrub or resilver to complete
// more quickly, but reads and writes to have higher latency and lower
// throughput.
uint32_t zfs_vdev_sync_read_min_active = 10;
uint32_t zfs_vdev_sync_read_max_active = 10;
uint32_t zfs_vdev_sync_write_min_active = 10;
uint32_t zfs_vdev_sync_write_max_active = 10;
uint32_t zfs_vdev_async_read_min_active = 1;
uint32_t zfs_vdev_async_read_max_active = 3;
uint32_t zfs_vdev_async_write_min_active = 1;
uint32_t zfs_vdev_async_write_max_active = 10;
uint32_t zfs_vdev_scrub_min_active = 1;
uint32_t zfs_vdev_scrub_max_active = 2;

// When the pool has less than zfs_vdev_async_write_active_min_dirty_percent
// dirty data, use zfs_vdev_async_write_min_active.  When it has more than
// zfs_vdev_async_write_active_max_dirty_percent, use
// zfs_vdev_async_write_max_active. The value is linearly interpolated
// between min and max.
int zfs_vdev_async_write_active_min_dirty_percent = 30;
int zfs_vdev_async_write_active_max_dirty_percent = 60;

// To reduce IOPs, we aggregate small adjacent IOs into one large IO.
// For read IOs, we also aggregate across small adjacency gaps; for writes
// we include spans of optional IOs to aid aggregation at the disk even when
// they aren't able to help us aggregate at this level.
int zfs_vdev_aggregation_limit = SPA_OLD_MAXBLOCKSIZE;
int zfs_vdev_read_gap_limit = 32 << 10;
int zfs_vdev_write_gap_limit = 4 << 10;

fn vdev_queue_offset_compare(const void *x1, const void *x2) -> i32 {
    const zio_t *z1 = x1;
    const zio_t *z2 = x2;

    if z1.offset < z2.offset {
        return -1;
    }
    if z1.offset > z2.offset {
        return 1;
    }

    if z1 < z2 {
        return -1;
    }
    if z1 > z2 {
        return 1;
    }

    return 0;
}

static inline avl_tree_t *
vdev_queue_class_tree(vdev_queue_t *vq, zio_priority_t p)
{
    return (&vq->vq_class[p].vqc_queued_tree);
}

static inline avl_tree_t *
vdev_queue_type_tree(vdev_queue_t *vq, zio_type_t t)
{
    assert!(t == ZIO_TYPE_READ || t == ZIO_TYPE_WRITE);
    if t == ZIO_TYPE_READ {
        return &vq->vq_read_offset_tree;
    } else {
        return &vq->vq_write_offset_tree;
    }
}

int
vdev_queue_timestamp_compare(const void *x1, const void *x2)
{
    const zio_t *z1 = x1;
    const zio_t *z2 = x2;

    if (z1->io_timestamp < z2->io_timestamp)
        return (-1);
    if (z1->io_timestamp > z2->io_timestamp)
        return (1);

    if (z1 < z2)
        return (-1);
    if (z1 > z2)
        return (1);

    return (0);
}

static int
vdev_queue_class_min_active(zio_priority_t p)
{
    switch (p) {
    case ZIO_PRIORITY_SYNC_READ:
        return (zfs_vdev_sync_read_min_active);
    case ZIO_PRIORITY_SYNC_WRITE:
        return (zfs_vdev_sync_write_min_active);
    case ZIO_PRIORITY_ASYNC_READ:
        return (zfs_vdev_async_read_min_active);
    case ZIO_PRIORITY_ASYNC_WRITE:
        return (zfs_vdev_async_write_min_active);
    case ZIO_PRIORITY_SCRUB:
        return (zfs_vdev_scrub_min_active);
    default:
        panic("invalid priority %u", p);
        return (0);
    }
}

static int
vdev_queue_max_async_writes(spa_t *spa)
{
    int writes;
    uint64_t dirty = spa->spa_dsl_pool->dp_dirty_total;
    uint64_t min_bytes = zfs_dirty_data_max *
        zfs_vdev_async_write_active_min_dirty_percent / 100;
    uint64_t max_bytes = zfs_dirty_data_max *
        zfs_vdev_async_write_active_max_dirty_percent / 100;

    // Sync tasks correspond to interactive user actions. To reduce the
    // execution time of those actions we push data out as fast as possible.
    if (spa_has_pending_synctask(spa)) {
        return zfs_vdev_async_write_max_active;
    }

    if dirty < min_bytes {
        return zfs_vdev_async_write_min_active;
    }
    if dirty > max_bytes {
        return zfs_vdev_async_write_max_active;
    }

    // linear interpolation:
    // slope = (max_writes - min_writes) / (max_bytes - min_bytes)
    // move right by min_bytes
    // move up by min_writes
    writes = (dirty - min_bytes) *
             (zfs_vdev_async_write_max_active - zfs_vdev_async_write_min_active) /
             (max_bytes - min_bytes) + zfs_vdev_async_write_min_active;
    assert!(writes >= zfs_vdev_async_write_min_active);
    assert!(writes <= zfs_vdev_async_write_max_active);
    return (writes);
}

fn vdev_queue_class_max_active(spa_t *spa, p: zio::Priority) -> int {
    match p {
        zio::Priority::SyncRead =>  zfs_vdev_sync_read_max_active,
        zio::Priority::SyncWrite => zfs_vdev_sync_write_max_active,
        zio::Priority::AsyncRead => zfs_vdev_async_read_max_active,
        zio::Priority::AsyncWrite => vdev_queue_max_async_writes(spa),
        zio::Priority::Scrub => zfs_vdev_scrub_max_active,
        _ => {
            panic!("invalid priority {}", p);
            0
        }
    }
}

// Return the IO class to issue from, or ZIO_PRIORITY_MAX_QUEUEABLE if
// there is no eligible class.
static zio_priority_t
vdev_queue_class_to_issue(vdev_queue_t *vq)
{
    spa_t *spa = vq->vq_vdev->vdev_spa;
    zio_priority_t p;

    if (avl_numnodes(&vq->vq_active_tree) >= zfs_vdev_max_active)
        return (ZIO_PRIORITY_NUM_QUEUEABLE);

    for (p = 0; p < ZIO_PRIORITY_NUM_QUEUEABLE; p++) {
        if (avl_numnodes(vdev_queue_class_tree(vq, p)) > 0 &&
            vq->vq_class[p].vqc_active <
            vdev_queue_class_min_active(p))
            return (p);
    }

    // If we haven't found a queue, look for one that hasn't reached its
    // maximum # outstanding IOs.
    for (p = 0; p < ZIO_PRIORITY_NUM_QUEUEABLE; p++) {
        if (avl_numnodes(vdev_queue_class_tree(vq, p)) > 0 &&
            vq->vq_class[p].vqc_active <
            vdev_queue_class_max_active(spa, p))
            return (p);
    }

    return (ZIO_PRIORITY_NUM_QUEUEABLE);
}

void
vdev_queue_init(vdev_t *vd)
{
    vdev_queue_t *vq = &vd->vdev_queue;
    zio_priority_t p;

    mutex_init(&vq->vq_lock, NULL, MUTEX_DEFAULT, NULL);
    vq->vq_vdev = vd;
    taskq_init_ent(&vd->vdev_queue.vq_io_search.io_tqent);

    avl_create(&vq->vq_active_tree, vdev_queue_offset_compare,
        sizeof (zio_t), offsetof(struct zio, io_queue_node));
    avl_create(vdev_queue_type_tree(vq, ZIO_TYPE_READ),
        vdev_queue_offset_compare, sizeof (zio_t),
        offsetof(struct zio, io_offset_node));
    avl_create(vdev_queue_type_tree(vq, ZIO_TYPE_WRITE),
        vdev_queue_offset_compare, sizeof (zio_t),
        offsetof(struct zio, io_offset_node));

    for (p = 0; p < ZIO_PRIORITY_NUM_QUEUEABLE; p++) {
        int (*compfn) (const void *, const void *);

        // The synchronous IO queues are dispatched in FIFO rather
        // than LBA order. This provides more consistent latency for
        // these IOs.
        if (p == ZIO_PRIORITY_SYNC_READ || p == ZIO_PRIORITY_SYNC_WRITE)
            compfn = vdev_queue_timestamp_compare;
        else
            compfn = vdev_queue_offset_compare;
        avl_create(vdev_queue_class_tree(vq, p), compfn,
            sizeof (zio_t), offsetof(struct zio, io_queue_node));
    }
}

void
vdev_queue_fini(vdev_t *vd)
{
    vdev_queue_t *vq = &vd->vdev_queue;
    zio_priority_t p;

    for (p = 0; p < ZIO_PRIORITY_NUM_QUEUEABLE; p++)
        avl_destroy(vdev_queue_class_tree(vq, p));
    avl_destroy(&vq->vq_active_tree);
    avl_destroy(vdev_queue_type_tree(vq, ZIO_TYPE_READ));
    avl_destroy(vdev_queue_type_tree(vq, ZIO_TYPE_WRITE));

    mutex_destroy(&vq->vq_lock);
}

static void
vdev_queue_io_add(vdev_queue_t *vq, zio_t *zio)
{
    spa_t *spa = zio->io_spa;
    spa_stats_history_t *ssh = &spa->spa_stats.io_history;

    assert!(zio->io_priority, <, ZIO_PRIORITY_NUM_QUEUEABLE);
    avl_add(vdev_queue_class_tree(vq, zio->io_priority), zio);
    avl_add(vdev_queue_type_tree(vq, zio->io_type), zio);

    if (ssh->kstat != NULL) {
        mutex_enter(&ssh->lock);
        kstat_waitq_enter(ssh->kstat->ks_data);
        mutex_exit(&ssh->lock);
    }
}

static void
vdev_queue_io_remove(vdev_queue_t *vq, zio_t *zio)
{
    spa_t *spa = zio->io_spa;
    spa_stats_history_t *ssh = &spa->spa_stats.io_history;

    assert!(zio->io_priority, <, ZIO_PRIORITY_NUM_QUEUEABLE);
    avl_remove(vdev_queue_class_tree(vq, zio->io_priority), zio);
    avl_remove(vdev_queue_type_tree(vq, zio->io_type), zio);

    if (ssh->kstat != NULL) {
        mutex_enter(&ssh->lock);
        kstat_waitq_exit(ssh->kstat->ks_data);
        mutex_exit(&ssh->lock);
    }
}

static void
vdev_queue_pending_add(vdev_queue_t *vq, zio_t *zio)
{
    spa_t *spa = zio->io_spa;
    spa_stats_history_t *ssh = &spa->spa_stats.io_history;

    ASSERT(MUTEX_HELD(&vq->vq_lock));
    assert!(zio->io_priority, <, ZIO_PRIORITY_NUM_QUEUEABLE);
    vq->vq_class[zio->io_priority].vqc_active++;
    avl_add(&vq->vq_active_tree, zio);

    if (ssh->kstat != NULL) {
        mutex_enter(&ssh->lock);
        kstat_runq_enter(ssh->kstat->ks_data);
        mutex_exit(&ssh->lock);
    }
}

static void
vdev_queue_pending_remove(vdev_queue_t *vq, zio_t *zio)
{
    spa_t *spa = zio->io_spa;
    spa_stats_history_t *ssh = &spa->spa_stats.io_history;

    ASSERT(MUTEX_HELD(&vq->vq_lock));
    assert!(zio->io_priority, <, ZIO_PRIORITY_NUM_QUEUEABLE);
    vq->vq_class[zio->io_priority].vqc_active--;
    avl_remove(&vq->vq_active_tree, zio);

    if (ssh->kstat != NULL) {
        kstat_io_t *ksio = ssh->kstat->ks_data;

        mutex_enter(&ssh->lock);
        kstat_runq_exit(ksio);
        if (zio->io_type == ZIO_TYPE_READ) {
            ksio->reads++;
            ksio->nread += zio->io_size;
        } else if (zio->io_type == ZIO_TYPE_WRITE) {
            ksio->writes++;
            ksio->nwritten += zio->io_size;
        }
        mutex_exit(&ssh->lock);
    }
}

fn vdev_queue_agg_io_done(aio: &mut Zio) {
    if (aio.zio_type == ZIO_TYPE_READ) {
        zio_t *pio;
        while (pio = zio_walk_parents(aio)) != NULL {
            bcopy(aio.data + (pio.offset - aio.offset), pio.data, pio.size);
        }
    }

    zio_buf_free(aio.data, aio.size);
}

// Compute the range spanned by two IOs, which is the endpoint of the last
// (lio->io_offset + lio->io_size) minus start of the first (fio->io_offset).
// Conveniently, the gap between fio and lio is given by -IO_SPAN(lio, fio);
// thus fio and lio are adjacent if and only if IO_SPAN(lio, fio) == 0.
#define    IO_SPAN(fio, lio) ((lio)->io_offset + (lio)->io_size - (fio)->io_offset)
#define    IO_GAP(fio, lio) (-IO_SPAN(lio, fio))

static zio_t *
vdev_queue_aggregate(vdev_queue_t *vq, zio_t *zio)
{
    zio_t *first, *last, *aio, *dio, *mandatory, *nio;
    uint64_t maxgap = 0;
    uint64_t size;
    boolean_t stretch = B_FALSE;
    avl_tree_t *t = vdev_queue_type_tree(vq, zio->io_type);
    enum zio_flag flags = zio->io_flags & ZIO_FLAG_AGG_INHERIT;

    if (zio->io_flags & ZIO_FLAG_DONT_AGGREGATE)
        return (NULL);

    // Prevent users from setting the zfs_vdev_aggregation_limit
    // tuning larger than SPA_MAXBLOCKSIZE.
    zfs_vdev_aggregation_limit =
        MIN(zfs_vdev_aggregation_limit, SPA_MAXBLOCKSIZE);

    first = last = zio;

    if (zio->io_type == ZIO_TYPE_READ)
        maxgap = zfs_vdev_read_gap_limit;

    // We can aggregate IOs that are sufficiently adjacent and of
    // the same flavor, as expressed by the AGG_INHERIT flags.
    // The latter requirement is necessary so that certain
    // attributes of the IO, such as whether it's a normal IO
    // or a scrub/resilver, can be preserved in the aggregate.
    // We can include optional IOs, but don't allow them
    // to begin a range as they add no benefit in that situation.

    // We keep track of the last non-optional IO.
    mandatory = (first->io_flags & ZIO_FLAG_OPTIONAL) ? NULL : first;

    // Walk backwards through sufficiently contiguous IOs
    // recording the last non-option IO.
    while ((dio = AVL_PREV(t, first)) != NULL &&
        (dio->io_flags & ZIO_FLAG_AGG_INHERIT) == flags &&
        IO_SPAN(dio, last) <= zfs_vdev_aggregation_limit &&
        IO_GAP(dio, first) <= maxgap) {
        first = dio;
        if (mandatory == NULL && !(first->io_flags & ZIO_FLAG_OPTIONAL))
            mandatory = first;
    }

    // Skip any initial optional IOs.
    while ((first->io_flags & ZIO_FLAG_OPTIONAL) && first != last) {
        first = AVL_NEXT(t, first);
        ASSERT(first != NULL);
    }


    // Walk forward through sufficiently contiguous IOs.
    while ((dio = AVL_NEXT(t, last)) != NULL &&
        (dio->io_flags & ZIO_FLAG_AGG_INHERIT) == flags &&
        IO_SPAN(first, dio) <= zfs_vdev_aggregation_limit &&
        IO_GAP(last, dio) <= maxgap) {
        last = dio;
        if (!(last->io_flags & ZIO_FLAG_OPTIONAL))
            mandatory = last;
    }

    // Now that we've established the range of the IO aggregation
    // we must decide what to do with trailing optional IOs.
    // For reads, there's nothing to do. While we are unable to
    // aggregate further, it's possible that a trailing optional
    // IO would allow the underlying device to aggregate with
    // subsequent IOs. We must therefore determine if the next
    // non-optional IO is close enough to make aggregation
    // worthwhile.
    if (zio->io_type == ZIO_TYPE_WRITE && mandatory != NULL) {
        zio_t *nio = last;
        while ((dio = AVL_NEXT(t, nio)) != NULL &&
            IO_GAP(nio, dio) == 0 &&
            IO_GAP(mandatory, dio) <= zfs_vdev_write_gap_limit) {
            nio = dio;
            if (!(nio->io_flags & ZIO_FLAG_OPTIONAL)) {
                stretch = B_TRUE;
                break;
            }
        }
    }

    if (stretch) {
        // This may be a no-op.
        dio = AVL_NEXT(t, last);
        dio->io_flags &= ~ZIO_FLAG_OPTIONAL;
    } else {
        while (last != mandatory && last != first) {
            ASSERT(last->io_flags & ZIO_FLAG_OPTIONAL);
            last = AVL_PREV(t, last);
            ASSERT(last != NULL);
        }
    }

    if (first == last)
        return (NULL);

    size = IO_SPAN(first, last);
    assert!(size, <=, zfs_vdev_aggregation_limit);

    aio = zio_vdev_delegated_io(first->io_vd, first->io_offset,
        zio_buf_alloc(size), size, first->io_type, zio->io_priority,
        flags | ZIO_FLAG_DONT_CACHE | ZIO_FLAG_DONT_QUEUE,
        vdev_queue_agg_io_done, NULL);
    aio->io_timestamp = first->io_timestamp;

    nio = first;
    do {
        dio = nio;
        nio = AVL_NEXT(t, dio);
        assert!(dio->io_type, ==, aio->io_type);

        if (dio->io_flags & ZIO_FLAG_NODATA) {
            assert!(dio->io_type, ==, ZIO_TYPE_WRITE);
            bzero((char *)aio->io_data + (dio->io_offset -
                aio->io_offset), dio->io_size);
        } else if (dio->io_type == ZIO_TYPE_WRITE) {
            bcopy(dio->io_data, (char *)aio->io_data +
                (dio->io_offset - aio->io_offset),
                dio->io_size);
        }

        zio_add_child(dio, aio);
        vdev_queue_io_remove(vq, dio);
        zio_vdev_io_bypass(dio);
        zio_execute(dio);
    } while (dio != last);

    return (aio);
}

fn vdev_queue_io_to_issue(vdev_queue_t *vq) -> Option<Zio> {
    zio_t *zio, *aio;
    zio_priority_t p;
    avl_index_t idx;
    avl_tree_t *tree;

again:
    ASSERT(MUTEX_HELD(&vq->vq_lock));

    p = vdev_queue_class_to_issue(vq);

    if (p == ZIO_PRIORITY_NUM_QUEUEABLE) {
        // No eligible queued IOs
        return (NULL);
    }

    // For LBA-ordered queues (async / scrub), issue the IO which follows
    // the most recently issued IO in LBA (offset) order.
    //
    // For FIFO queues (sync), issue the IO with the lowest timestamp.
    tree = vdev_queue_class_tree(vq, p);
    vq->vq_io_search.io_timestamp = 0;
    vq->vq_io_search.io_offset = vq->vq_last_offset + 1;
    //VERIFY(avl_find(tree, &vq->vq_io_search, &idx) == NULL);
    zio = avl_nearest(tree, idx, AVL_AFTER);
    if (zio == NULL)
        zio = avl_first(tree);
    assert!(zio->io_priority == p);

    aio = vdev_queue_aggregate(vq, zio);
    if (aio != NULL)
        zio = aio;
    else
        vdev_queue_io_remove(vq, zio);

    // If the IO is or was optional and therefore has no data, we need to
    // simply discard it. We need to drop the vdev queue's lock to avoid a
    // deadlock that we could encounter since this IO will complete
    // immediately.
    if (zio->io_flags & ZIO_FLAG_NODATA) {
        mutex_exit(&vq->vq_lock);
        zio_vdev_io_bypass(zio);
        zio_execute(zio);
        mutex_enter(&vq->vq_lock);
        goto again;
    }

    vdev_queue_pending_add(vq, zio);
    vq->vq_last_offset = zio->io_offset;

    return (zio);
}

pub fn vdev_queue_io(zio_t *zio) -> Option<Zio> {
    vdev_queue_t *vq = &zio.vd.vdev_queue;

    if zio->io_flags & ZIO_FLAG_DONT_QUEUE != 0 {
        return zio;
    }

    // Children IOs inherent their parent's priority, which might
    // not match the child's IO type. Fix it up here.
    if zio.zio_type == ZIO_TYPE_READ {
        if zio->io_priority != ZIO_PRIORITY_SYNC_READ &&
            zio->io_priority != ZIO_PRIORITY_ASYNC_READ &&
            zio->io_priority != ZIO_PRIORITY_SCRUB
        {
            zio->io_priority = ZIO_PRIORITY_ASYNC_READ;
        }
    } else {
        assert!(zio.zio_type == ZIO_TYPE_WRITE);
        if (zio.priority != ZIO_PRIORITY_SYNC_WRITE &&
            zio.priority != ZIO_PRIORITY_ASYNC_WRITE)
            zio.priority = ZIO_PRIORITY_ASYNC_WRITE;
    }

    zio->io_flags |= ZIO_FLAG_DONT_CACHE | ZIO_FLAG_DONT_QUEUE;

    mutex_enter(&vq->vq_lock);
    zio.timestamp = gethrtime();
    vdev_queue_io_add(vq, zio);
    let nio = vdev_queue_io_to_issue(vq);
    mutex_exit(&vq->vq_lock);

    if let Some(nio) = nio {
        if nio.done == vdev_queue_agg_io_done {
            nio.no_wait();
            return None;
        }
    }

    nio
}

fn vdev_queue_io_done(zio_t *zio) {
    vdev_queue_t *vq = &zio->io_vd->vdev_queue;
    zio_t *nio;

    if zio_injection_enabled {
        delay(SEC_TO_TICK(zio_handle_io_delay(zio)));
    }

    mutex_enter(&vq->vq_lock);

    vdev_queue_pending_remove(vq, zio);

    zio.delta = gethrtime() - zio.timestamp;
    vq.io_complete_ts = gethrtime();
    vq.io_delta_ts = vq.io_complete_ts - zio.timestamp;

    while (nio = vdev_queue_io_to_issue(vq)) != NULL {
        mutex_exit(&vq->vq_lock);
        if (nio.done == vdev_queue_agg_io_done) {
            nio.no_wait();
        } else {
            zio_vdev_io_reissue(nio);
            nio.execute();
        }
        mutex_enter(&vq.lock);
    }

    mutex_exit(&vq->vq_lock);
}
