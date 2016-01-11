use std::cmp;
// use std::collections::VecDeque;
// use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

use super::zfs;

const TQENT_FLAG_PREALLOC: u64 = 0x1; // taskq_dispatch_ent used

const TASKQ_PREPOPULATE: u64 = 0x0001;
const TASKQ_CPR_SAFE: u64 = 0x0002; // Use CPR safe protocol
const TASKQ_DYNAMIC: u64 = 0x0004; // Use dynamic thread scheduling
const TASKQ_THREADS_CPU_PCT: u64 = 0x0008; // Scale # threads by # cpus
const TASKQ_DC_BATCH: u64 = 0x0010; // Mark threads as batch

// const TQ_SLEEP: u64 = KM_SLEEP; // Can block for memory
// const TQ_NOSLEEP: u64 = KM_NOSLEEP; // Cannot block for memory; may fail
const TQ_NOQUEUE: u64 = 0x02; // Do not enqueue if can't dispatch
const TQ_FRONT: u64 = 0x08; // Queue in front

const TASKQ_ACTIVE: u64 = 0x00010000;

pub type TaskFn = Box<FnMut()>;

pub struct Taskq {
    name: String,
    // kmutex_t lock,
    // krwlock_t threadlock,
    // kcondvar_t dispatch_cv,
    // kcondvar_t wait_cv,*/
    // threads: Vec<Sender<Task>>,
    flags: u64,
    active: u16,
    num_threads: u16,
    num_alloc: u64,
    min_alloc: u64,
    max_alloc: u64,
    next_task_id: usize,
    // kcondvar_t max_alloc_cv,
    max_alloc_wait: i64, /* taskq_ent_t *freelist,
                          * task_queue: VecDeque<Task>, */
}

impl Taskq {
    pub fn new(name: String,
               mut num_threads: u16,
               min_alloc: u64,
               max_alloc: u64,
               flags: u64)
               -> Self {
        // taskq_t *tq = kmem_zalloc(sizeof (taskq_t), KM_SLEEP);

        // if flags & TASKQ_THREADS_CPU_PCT != 0 {
        // int pct;
        // assert!(num_threads >= 0);
        // assert!(num_threads <= 100);
        // pct = cmp::min(num_threads, 100);
        // pct = cmp::max(pct, 0);
        //
        // num_threads = (sysconf(_SC_NPROCESSORS_ONLN) * pct) / 100;
        // num_threads = cmp::max(num_threads, 1);    /* need at least 1 thread */
        // } else {
        // assert!(num_threads >= 1);
        // }

        // rw_init(&tq.threadlock, NULL, RW_DEFAULT, NULL);
        // mutex_init(&tq.lock, NULL, MUTEX_DEFAULT, NULL);
        // cv_init(&tq.dispatch_cv, NULL, CV_DEFAULT, NULL);
        // cv_init(&tq.wait_cv, NULL, CV_DEFAULT, NULL);
        // cv_init(&tq.max_alloc_cv, NULL, CV_DEFAULT, NULL);
        // tq.task.next: &tq.task;
        // tq.task.prev: &tq.task;

        // if flags & TASKQ_PREPOPULATE != 0 {
        // mutex_enter(&tq.lock);
        // while (min_alloc-- > 0)
        // task_free(tq, task_alloc(tq, KM_SLEEP));
        // mutex_exit(&tq.lock);
        // }

        // let mut threads = Vec::new();
        // for _ in 0..num_threads {
        // let (task_t, task_r) = channel();
        // threads.push(task_t);
        // thread::spawn(|| { taskq_thread(task_r) });
        // tq.thread_list[t] = thread_create(NULL, 0, taskq_thread, tq, TS_RUN, NULL, 0, pri);
        // VERIFIY(tq.thread_list[t]);
        // }

        Taskq {
            name: name,
            // threads: threads,
            flags: flags | TASKQ_ACTIVE,
            active: num_threads,
            num_threads: num_threads,
            num_alloc: 0,
            min_alloc: min_alloc,
            max_alloc: max_alloc,
            next_task_id: 0,
            max_alloc_wait: 0, // task_queue: VecDeque::new(),
        }
    }

    // fn alloc_task(&mut self, tqflags: u64) -> Self {
    // taskq_ent_t *t;
    //
    // loop {
    // if (t = self.freelist) != NULL && self.num_alloc >= self.min_alloc {
    // There's a free Task in the free_list
    // assert!(t.flags & TQENT_FLAG_PREALLOC == 0);
    // self.freelist = t.next;
    // } else {
    // if (self.num_alloc >= self.max_alloc) {
    // if tqflags & KM_SLEEP == 0 {
    // return NULL;
    // }
    //
    // We don't want to exceed max_alloc, but we can't
    // wait for other tasks to complete (and thus free up
    // task structures) without risking deadlock with
    // the caller.  So, we just delay for one second
    // to throttle the allocation rate. If we have tasks
    // complete before one second timeout expires then
    // taskq_ent_free will signal us and we will
    // immediately retry the allocation.
    // self.max_alloc_wait += 1;
    // let rv = cv_timedwait(&self.max_alloc_cv, &self.lock, ddi_get_lbolt() + hz);
    // self.max_alloc_wait -= 1;
    // if rv > 0 {
    // continue;
    // }
    // }
    // mutex_exit(&self.lock);
    //
    // t = kmem_alloc(sizeof (taskq_ent_t), tqflags);
    //
    // mutex_enter(&self.lock);
    // if t != NULL {
    // Make sure we start without any flags
    // t.flags = 0;
    // self.num_alloc++;
    // }
    // }
    //
    // break;
    // }
    // return t;
    // }

    // fn task_free(taskq_t *tq, taskq_ent_t *t) {
    // if (tq->tq_nalloc <= tq->tq_min_alloc) {
    // t->tqent_next = tq->tq_freelist;
    // tq->tq_freelist = t;
    // } else {
    // tq->tq_nalloc--;
    // mutex_exit(&tq->tq_lock);
    // kmem_free(t, sizeof (taskq_ent_t));
    // mutex_enter(&tq->tq_lock);
    // }
    //
    // if (tq->tq_max_alloc_wait) {
    // cv_signal(&tq->tq_max_alloc_cv);
    // }
    // }

    fn taskq_dispatch(&mut self, func: TaskFn, flags: u64) -> TaskId {
        // self.threads[0].send(Task { func: func, flags: flags });
        let index = self.next_task_id;
        self.next_task_id += 1;
        TaskId(index)
    }

    // fn taskq_dispatch(&mut self, func: TaskFn, flags: u64) -> TaskId {
    // taskq_ent_t *t;
    //
    // if taskq_now {
    // func(arg);
    // return 1;
    // }
    //
    // mutex_enter(&self.lock);
    // assert!(self.flags & TASKQ_ACTIVE);
    // if (t = self.alloc_task(tqflags)) == NULL {
    // mutex_exit(&self.lock);
    // return 0;
    // }
    // if tqflags & TQ_FRONT != 0 {
    // t.next = self.task.next;
    // t.prev = &self.task;
    // } else {
    // t.next = &self.task;
    // t.prev = self.task.prev;
    // }
    // t.next.prev = t;
    // t.prev.next = t;
    // t.func = func;
    // t.flags = 0;
    // cv_signal(&self.dispatch_cv);
    // mutex_exit(&self.lock);
    // return 1;
    // }
    //
    // taskqid_t
    // taskq_dispatch_delay(taskq_t *tq, task_func_t func, uint_t tqflags,
    // clock_t expire_time)
    // {
    // return 0;
    // }

    // pub fn empty_ent(&self) -> bool {
    // self.next == NULL
    // }

    // fn taskq_init_ent(taskq_ent_t *t) {
    // t.next = NULL;
    // t.prev = NULL;
    // t.func = NULL;
    // t.flags = 0;
    // }

    // fn taskq_dispatch_ent(taskq_t *tq, task_func_t func, uint_t flags, taskq_ent_t *t) {
    // assert!(func != NULL);
    //
    // Mark it as a prealloc'd task.  This is important
    // to ensure that we don't free it later.
    // t.flags |= TQENT_FLAG_PREALLOC;
    // Enqueue the task to the underlying queue.
    // mutex_enter(&tq.lock);
    //
    // if (flags & TQ_FRONT) {
    // t.next = tq.task.next;
    // t.prev = &tq.task;
    // } else {
    // t.next = &tq.task;
    // t.prev = tq.task.prev;
    // }
    // t.next.prev = t;
    // t.prev.next = t;
    // t.func = func;
    // cv_signal(&tq.dispatch_cv);
    // mutex_exit(&tq.lock);
    // }

    // fn wait(&self) {
    // mutex_enter(&tq.lock);
    // while tq.task.next != &tq.task || tq.active > 0 {
    // cv_wait(&tq.wait_cv, &tq.lock);
    // }
    // mutex_exit(&tq.lock);
    // }
    //
    // fn wait_id(&self, id: TaskId) {
    // self.wait();
    // }
    //
    // fn wait_outstanding(&self, id: TaskId) {
    // self.wait();
    // }
    //
    // fn destroy(&mut self) {
    // int num_threads = tq->tq_num_threads;
    //
    // taskq_wait(tq);
    //
    // mutex_enter(&tq->tq_lock);
    //
    // tq->tq_flags &= ~TASKQ_ACTIVE;
    // cv_broadcast(&tq->tq_dispatch_cv);
    //
    // while tq->tq_num_threads > 0 {
    // cv_wait(&tq->tq_wait_cv, &tq->tq_lock);
    // }
    //
    // tq.min_alloc = 0;
    // while (tq.num_alloc != 0) {
    // ASSERT(tq->tq_freelist != NULL);
    // task_free(tq, task_alloc(tq, KM_SLEEP));
    // }
    //
    // mutex_exit(&tq->tq_lock);
    //
    // kmem_free(tq->tq_thread_list, num_threads * sizeof (kthread_t *));
    //
    // rw_destroy(&tq->tq_threadlock);
    // mutex_destroy(&tq->tq_lock);
    // cv_destroy(&tq->tq_dispatch_cv);
    // cv_destroy(&tq->tq_wait_cv);
    // cv_destroy(&tq->tq_max_alloc_cv);
    //
    // kmem_free(tq, sizeof (taskq_t));
    // }
    //
    // pub fn member(&self, thread_id: ThreadId) -> bool {
    // for i in 0..self.num_threads {
    // if self.thread_list[i] == t {
    // return true;
    // }
    // }
    //
    // false
    // }

    pub fn cancel_id(&mut self, id: TaskId) -> zfs::Result<()> {
        Err(zfs::Error::NoEntity)
    }
}

// fn system_taskq_init() {
// system_taskq = taskq_create("system_taskq", 64, maxclsyspri, 4, 512,
// TASKQ_DYNAMIC | TASKQ_PREPOPULATE);
// }
//
// fn system_taskq_fini() {
// taskq_destroy(system_taskq);
// system_taskq = NULL; // defensive
// }

//-------------------------------------------------------------------------------------------------//

pub struct TaskId(usize);

struct Task {
    // taskq_ent *next;
    // taskq_ent *prev;
    func: Box<FnMut()>,
    flags: u64,
}

//-------------------------------------------------------------------------------------------------//

// fn taskq_thread(task_r: Receiver<Task>) {
// while let Ok(task) = task_r.recv() {
// (task.func)();
// }
// }

// fn taskq_thread(task_r: Receiver<Task>) {
// taskq_t *tq = arg;
// taskq_ent_t *t;
//
// mutex_enter(&tq.lock);
// while tq.flags & TASKQ_ACTIVE != 0 {
// if (t = tq.task.next) == &tq.task {
// tq.active -= 1;
// if tq.active == 0 {
// cv_broadcast(&tq.wait_cv);
// }
// cv_wait(&tq.dispatch_cv, &tq.lock);
// tq.active++;
// continue;
// }
// t.prev.next = t.next;
// t.next.prev = t.prev;
// t.next = NULL;
// t.prev = NULL;
// mutex_exit(&tq.lock);
//
// rw_enter(&tq.threadlock, RW_READER);
// t.func(t.arg);
// rw_exit(&tq.threadlock);
//
// mutex_enter(&tq.lock);
// if !t.flags & TQENT_FLAG_PREALLOC != 0 {
// task_free(tq, t);
// }
// }
// tq.num_threads--;
// cv_broadcast(&tq.wait_cv);
// mutex_exit(&tq.lock);
// thread_exit();
// }
