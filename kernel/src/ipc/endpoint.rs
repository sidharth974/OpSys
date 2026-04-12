use alloc::collections::VecDeque;
use alloc::vec::Vec;
use spin::Mutex;
use super::message::Message;
use crate::proc::scheduler::SCHEDULER;

/// Global endpoint table.
pub static ENDPOINTS: Mutex<EndpointTable> = Mutex::new(EndpointTable::new());

/// An IPC endpoint: a rendezvous point for synchronous message passing.
///
/// seL4-style: a sender blocks until a receiver is ready, and vice versa.
/// When both are present, the message is transferred directly.
pub struct Endpoint {
    /// Threads waiting to send (TID + their message).
    send_queue: VecDeque<(usize, Message)>,
    /// Threads waiting to receive (TID).
    recv_queue: VecDeque<usize>,
}

impl Endpoint {
    pub const fn new() -> Self {
        Self {
            send_queue: VecDeque::new(),
            recv_queue: VecDeque::new(),
        }
    }

    /// Send a message on this endpoint.
    /// If a receiver is waiting, transfer immediately and unblock it.
    /// Otherwise, block the sender.
    pub fn send(&mut self, sender_tid: usize, msg: Message) {
        if let Some(receiver_tid) = self.recv_queue.pop_front() {
            // A receiver is waiting — deliver the message directly
            // Store the message in the receiver's context (r12-r15 + stack)
            // For now, we use a simple message buffer approach
            unsafe {
                deliver_message(receiver_tid, &msg);
            }
            // Unblock the receiver
            SCHEDULER.lock().unblock(receiver_tid);
        } else {
            // No receiver — block the sender
            self.send_queue.push_back((sender_tid, msg));
            SCHEDULER.lock().block_current();
        }
    }

    /// Receive a message from this endpoint.
    /// If a sender is waiting, take its message immediately and unblock it.
    /// Otherwise, block the receiver.
    pub fn recv(&mut self, receiver_tid: usize) -> Option<Message> {
        if let Some((sender_tid, msg)) = self.send_queue.pop_front() {
            // A sender is waiting — take its message
            SCHEDULER.lock().unblock(sender_tid);
            Some(msg)
        } else {
            // No sender — block the receiver
            self.recv_queue.push_back(receiver_tid);
            SCHEDULER.lock().block_current();
            // When we're unblocked, the message has been delivered to us
            None // Caller should re-check after being unblocked
        }
    }
}

/// Endpoint table: global registry of all endpoints.
pub struct EndpointTable {
    endpoints: Vec<Option<Endpoint>>,
}

impl EndpointTable {
    pub const fn new() -> Self {
        Self {
            endpoints: Vec::new(),
        }
    }

    /// Create a new endpoint and return its ID.
    pub fn create(&mut self) -> usize {
        let id = self.endpoints.len();
        self.endpoints.push(Some(Endpoint::new()));
        id
    }

    /// Get a mutable reference to an endpoint.
    pub fn get_mut(&mut self, id: usize) -> Option<&mut Endpoint> {
        self.endpoints.get_mut(id)?.as_mut()
    }
}

/// Deliver a message to a blocked receiver thread.
/// In a full implementation, this would write to the thread's register save area.
/// For kernel threads, we use a per-thread message buffer.
///
/// # Safety
/// The receiver_tid must be a valid, blocked thread.
unsafe fn deliver_message(_receiver_tid: usize, _msg: &Message) {
    // TODO: In Phase 3, this will write to the thread's saved register context
    // so that when the thread resumes, it sees the message in its registers.
    // For now, kernel threads use a shared message buffer approach (see the test).
}
