use alloc::collections::VecDeque;
use alloc::vec::Vec;
use spin::Mutex;
use super::thread::{Thread, ThreadState, Context};
use super::context_switch;

/// Number of priority levels (0 = lowest, 31 = highest).
pub const NUM_PRIORITIES: usize = 32;

/// Priority bands for different workload types.
pub const PRIORITY_IDLE: u8 = 0;
pub const PRIORITY_BACKGROUND: u8 = 4;
pub const PRIORITY_NORMAL: u8 = 16;
pub const PRIORITY_SYSTEM: u8 = 20;
pub const PRIORITY_AI_BATCH: u8 = 24;
pub const PRIORITY_AI_REALTIME: u8 = 28;

/// Global scheduler instance.
pub static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

/// Perform a context switch. Called outside the scheduler lock.
///
/// # Safety
/// The old_ctx and new_ctx pointers must be valid and non-aliasing.
pub unsafe fn do_schedule() {
    let switch_info = {
        let mut sched = SCHEDULER.lock();
        sched.pick_next()
    };

    if let Some((old_ctx, new_ctx)) = switch_info {
        unsafe { context_switch::switch_context(old_ctx, new_ctx); }
    }
}

pub struct Scheduler {
    ready_queues: [Option<VecDeque<usize>>; NUM_PRIORITIES],
    ready_bitmap: u32,
    threads: Vec<Option<Thread>>,
    current_tid: usize,
    initialized: bool,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            ready_queues: [const { None }; NUM_PRIORITIES],
            ready_bitmap: 0,
            threads: Vec::new(),
            current_tid: 0,
            initialized: false,
        }
    }

    /// Initialize the scheduler.
    /// The current execution context (kernel_main) becomes thread 0.
    pub fn init(&mut self) {
        for queue in self.ready_queues.iter_mut() {
            *queue = Some(VecDeque::new());
        }

        self.threads.push(Some(Thread::boot_thread()));
        self.threads[0].as_mut().unwrap().state = ThreadState::Running;
        self.current_tid = 0;
        self.initialized = true;
    }

    /// Spawn a new thread and add it to the ready queue.
    pub fn spawn(&mut self, thread: Thread) -> usize {
        let tid = thread.tid;
        let priority = thread.priority as usize;

        while self.threads.len() <= tid {
            self.threads.push(None);
        }
        self.threads[tid] = Some(thread);

        self.enqueue(tid, priority);
        tid
    }

    fn enqueue(&mut self, tid: usize, priority: usize) {
        if let Some(ref mut queue) = self.ready_queues[priority] {
            queue.push_back(tid);
            self.ready_bitmap |= 1 << priority;
        }
    }

    fn dequeue_highest(&mut self) -> Option<usize> {
        if self.ready_bitmap == 0 {
            return None;
        }
        let priority = 31 - self.ready_bitmap.leading_zeros() as usize;
        if let Some(ref mut queue) = self.ready_queues[priority] {
            let tid = queue.pop_front();
            if queue.is_empty() {
                self.ready_bitmap &= !(1 << priority);
            }
            tid
        } else {
            None
        }
    }

    /// Pick the next thread and return raw context pointers for switching.
    /// Returns None if no switch is needed.
    /// The caller must perform the actual switch AFTER releasing the lock.
    fn pick_next(&mut self) -> Option<(*mut Context, *const Context)> {
        if !self.initialized {
            return None;
        }

        let old_tid = self.current_tid;

        // First, dequeue the next ready thread BEFORE re-enqueuing the current one.
        // This gives other threads a chance to run even at lower priorities,
        // because the current thread won't be in the queue competing with itself.
        let candidate = self.dequeue_highest();

        // Put the current thread back on the ready queue (after dequeue!)
        if let Some(ref mut old_thread) = self.threads[old_tid] {
            if old_thread.state == ThreadState::Running {
                old_thread.state = ThreadState::Ready;
                let priority = old_thread.priority as usize;
                self.enqueue(old_tid, priority);
            }
        }

        let new_tid = match candidate {
            Some(tid) => tid,
            None => {
                // Nothing else to run — stay on current
                // Remove it from the queue we just added it to
                if let Some(ref mut t) = self.threads[old_tid] {
                    t.state = ThreadState::Running;
                    let priority = t.priority as usize;
                    // Pop the thread we just re-enqueued
                    if let Some(ref mut queue) = self.ready_queues[priority] {
                        if let Some(pos) = queue.iter().position(|&x| x == old_tid) {
                            queue.remove(pos);
                            if queue.is_empty() {
                                self.ready_bitmap &= !(1 << priority);
                            }
                        }
                    }
                }
                return None;
            }
        };

        if new_tid == old_tid {
            if let Some(ref mut t) = self.threads[old_tid] {
                t.state = ThreadState::Running;
            }
            return None;
        }

        self.current_tid = new_tid;
        if let Some(ref mut t) = self.threads[new_tid] {
            t.state = ThreadState::Running;
        }

        let old_ctx = &raw mut self.threads[old_tid].as_mut().unwrap().context;
        let new_ctx = &raw const self.threads[new_tid].as_ref().unwrap().context;

        Some((old_ctx, new_ctx))
    }

    /// Block the current thread.
    pub fn block_current(&mut self) {
        if let Some(ref mut t) = self.threads[self.current_tid] {
            t.state = ThreadState::Blocked;
        }
    }

    /// Unblock a thread and put it back on the ready queue.
    pub fn unblock(&mut self, tid: usize) {
        if let Some(ref mut t) = self.threads[tid] {
            if t.state == ThreadState::Blocked {
                t.state = ThreadState::Ready;
                let priority = t.priority as usize;
                self.enqueue(tid, priority);
            }
        }
    }

    pub fn current_tid(&self) -> usize {
        self.current_tid
    }

    pub fn thread_ref(&self, tid: usize) -> Option<&Thread> {
        self.threads.get(tid)?.as_ref()
    }

    pub fn thread_mut(&mut self, tid: usize) -> Option<&mut Thread> {
        self.threads.get_mut(tid)?.as_mut()
    }
}
