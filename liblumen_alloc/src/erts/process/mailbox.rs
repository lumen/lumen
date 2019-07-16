use core::alloc::AllocErr;
use core::default::Default;
#[cfg(test)]
use core::fmt::{self, Debug};

use intrusive_collections::linked_list::Iter;
use intrusive_collections::{LinkedList, UnsafeRef};

use crate::borrow::CloneToProcess;
use crate::erts::message::{self, Message};
use crate::erts::process::alloc::HeapAlloc;
use crate::erts::term::Term;

#[derive(Debug)]
pub struct Mailbox {
    messages: LinkedList<message::Adapter>,
    len: usize,
    seen: isize,
}

impl Mailbox {
    pub fn iter(&self) -> Iter<message::Adapter> {
        self.messages.iter()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn mark_seen(&mut self) {
        self.seen = (self.len() as isize) - 1;
    }

    /// Pops the `message` out of the mailbox from the front of the queue.
    pub fn pop(&mut self) -> Option<UnsafeRef<Message>> {
        match self.messages.pop_front() {
            option_message @ Some(_) => {
                self.len -= 1;

                option_message
            }
            None => None,
        }
    }

    /// Puts `message` into mailbox at end of receive queue.
    pub fn push(&mut self, message: UnsafeRef<Message>) {
        self.messages.push_back(message);
        self.len += 1;
    }

    /// Pops the `message` out of the mailbox from the front of the queue AND clones it into
    /// `heap_guard` heap.
    pub fn receive<A: HeapAlloc>(&mut self, heap: &mut A) -> Option<Result<Term, AllocErr>> {
        self.messages.pop_front().map(|message| {
            if message.is_on_heap() {
                Ok(message.data())
            } else {
                message.data().clone_to_heap(heap).map_err(|error| {
                    self.messages.push_front(message);

                    error
                })
            }
        })
    }

    pub fn remove(&mut self, index: usize) -> Option<UnsafeRef<Message>> {
        let mut cursor_mut = self.messages.front_mut();
        let mut current_index = 0;
        let mut removed = None;

        while !cursor_mut.is_null() && (current_index <= index) {
            if current_index == index {
                removed = cursor_mut.remove();

                if (index as isize) <= self.seen {
                    self.seen -= 1;
                }

                break;
            }

            cursor_mut.move_next();
            current_index += 1;
        }

        removed
    }

    pub fn seen(&self) -> isize {
        self.seen
    }

    pub fn unmark_seen(&mut self) {
        self.seen = -1;
    }
}

#[cfg(test)]
impl Debug for Mailbox {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.messages)
    }
}

impl Default for Mailbox {
    fn default() -> Mailbox {
        Mailbox {
            messages: Default::default(),
            len: 0,
            seen: -1,
        }
    }
}