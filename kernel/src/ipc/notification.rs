use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};
use spin::Mutex;
use crate::proc::scheduler::SCHEDULER;

/// Global notification table.
pub static NOTIFICATIONS: Mutex<NotificationTable> = Mutex::new(NotificationTable::new());

/// A notification object: an asynchronous 32-bit bitmask signal.
///
/// Used for lightweight signaling (e.g., interrupt delivery to userspace drivers).
/// Multiple signals can be OR'd together without blocking.
pub struct Notification {
    /// The notification word: bits are OR'd in by signalers.
    word: AtomicU32,
    /// Thread waiting for this notification (at most one).
    waiting_tid: Option<usize>,
}

impl Notification {
    pub fn new() -> Self {
        Self {
            word: AtomicU32::new(0),
            waiting_tid: None,
        }
    }

    /// Signal (set bits) on this notification. Never blocks.
    pub fn signal(&mut self, bits: u32) {
        self.word.fetch_or(bits, Ordering::SeqCst);
        // If a thread is waiting, wake it up
        if let Some(tid) = self.waiting_tid.take() {
            SCHEDULER.lock().unblock(tid);
        }
    }

    /// Wait for any bits to be set. Returns the bits and clears them.
    /// If bits are already set, returns immediately.
    /// Otherwise, blocks until signaled.
    pub fn wait(&mut self, tid: usize) -> u32 {
        let bits = self.word.swap(0, Ordering::SeqCst);
        if bits != 0 {
            return bits;
        }
        // No bits set — block
        self.waiting_tid = Some(tid);
        SCHEDULER.lock().block_current();
        // When unblocked, read and clear
        self.word.swap(0, Ordering::SeqCst)
    }
}

/// Notification table: global registry of all notification objects.
pub struct NotificationTable {
    notifications: Vec<Option<Notification>>,
}

impl NotificationTable {
    pub const fn new() -> Self {
        Self {
            notifications: Vec::new(),
        }
    }

    pub fn create(&mut self) -> usize {
        let id = self.notifications.len();
        self.notifications.push(Some(Notification::new()));
        id
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut Notification> {
        self.notifications.get_mut(id)?.as_mut()
    }
}
