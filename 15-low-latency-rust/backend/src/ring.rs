use crate::error::EngineError;
use std::cell::UnsafeCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Shared<T> {
    slots: Box<[UnsafeCell<T>]>,
    mask: usize,
    head: AtomicUsize,
    tail: AtomicUsize,
}

unsafe impl<T: Send> Sync for Shared<T> {}

pub struct Producer<T> {
    shared: Arc<Shared<T>>,
}

pub struct Consumer<T> {
    shared: Arc<Shared<T>>,
}

pub fn ring<T: Copy + Default + Send>(
    capacity: usize,
) -> Result<(Producer<T>, Consumer<T>), EngineError> {
    if capacity < 2 || !capacity.is_power_of_two() {
        return Err(EngineError::InvalidConfig(format!(
            "ring capacity {capacity} must be a power of two >= 2"
        )));
    }
    let slots: Box<[UnsafeCell<T>]> = (0..capacity)
        .map(|_| UnsafeCell::new(T::default()))
        .collect();
    let shared = Arc::new(Shared {
        slots,
        mask: capacity - 1,
        head: AtomicUsize::new(0),
        tail: AtomicUsize::new(0),
    });
    Ok((
        Producer {
            shared: Arc::clone(&shared),
        },
        Consumer { shared },
    ))
}

impl<T> Shared<T> {
    fn capacity(&self) -> usize {
        self.mask + 1
    }

    fn len(&self) -> usize {
        let tail = self.tail.load(Ordering::Acquire);
        let head = self.head.load(Ordering::Acquire);
        tail.wrapping_sub(head)
    }
}

impl<T: Copy> Producer<T> {
    pub fn push(&mut self, value: T) -> bool {
        let s = &*self.shared;
        let tail = s.tail.load(Ordering::Relaxed);
        let head = s.head.load(Ordering::Acquire);
        if tail.wrapping_sub(head) == s.capacity() {
            return false;
        }
        unsafe {
            *s.slots[tail & s.mask].get() = value;
        }
        s.tail.store(tail.wrapping_add(1), Ordering::Release);
        true
    }

    pub fn len(&self) -> usize {
        self.shared.len()
    }
}

impl<T: Copy> Consumer<T> {
    pub fn pop(&mut self) -> Option<T> {
        let s = &*self.shared;
        let head = s.head.load(Ordering::Relaxed);
        let tail = s.tail.load(Ordering::Acquire);
        if head == tail {
            return None;
        }
        let value = unsafe { *s.slots[head & s.mask].get() };
        s.head.store(head.wrapping_add(1), Ordering::Release);
        Some(value)
    }

    pub fn len(&self) -> usize {
        self.shared.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn rejects_capacity_that_is_not_a_power_of_two_because_masking_would_alias_slots() {
        assert!(ring::<u32>(1000).is_err());
        assert!(ring::<u32>(0).is_err());
        assert!(ring::<u32>(1024).is_ok());
    }

    #[test]
    fn full_ring_refuses_pushes_so_unread_audio_is_never_overwritten() {
        let (mut p, mut c) = ring::<u32>(4).unwrap();
        for i in 0..4 {
            assert!(p.push(i));
        }
        assert!(!p.push(99));
        assert_eq!(p.len(), 4);
        assert_eq!(c.pop(), Some(0));
        assert!(p.push(4));
        assert_eq!(c.pop(), Some(1));
    }

    #[test]
    fn empty_ring_returns_none_so_the_output_can_count_an_underrun() {
        let (mut p, mut c) = ring::<u32>(8).unwrap();
        assert_eq!(c.len(), 0);
        assert_eq!(c.pop(), None);
        p.push(7);
        assert_eq!(c.pop(), Some(7));
        assert_eq!(c.pop(), None);
    }

    #[test]
    fn indices_wrap_around_many_times_without_reordering_samples() {
        let (mut p, mut c) = ring::<u32>(8).unwrap();
        let mut next_in = 0u32;
        let mut next_out = 0u32;
        for round in 0..1000 {
            let n = 1 + round % 7;
            for _ in 0..n {
                assert!(p.push(next_in));
                next_in += 1;
            }
            for _ in 0..n {
                assert_eq!(c.pop(), Some(next_out));
                next_out += 1;
            }
        }
        assert!(next_in > 8 * 100);
        assert_eq!(c.len(), 0);
    }

    #[test]
    fn two_threads_transfer_every_sample_exactly_once_and_in_order() {
        const TOTAL: u64 = 2_000_000;
        let (mut p, mut c) = ring::<u64>(256).unwrap();
        let producer = thread::spawn(move || {
            let mut i = 0u64;
            while i < TOTAL {
                if p.push(i) {
                    i += 1;
                } else {
                    std::hint::spin_loop();
                }
            }
        });
        let consumer = thread::spawn(move || {
            let mut expected = 0u64;
            let mut sum = 0u64;
            while expected < TOTAL {
                if let Some(v) = c.pop() {
                    assert_eq!(v, expected, "lost or duplicated sample");
                    sum += v;
                    expected += 1;
                } else {
                    std::hint::spin_loop();
                }
            }
            assert_eq!(c.pop(), None);
            sum
        });
        producer.join().unwrap();
        let sum = consumer.join().unwrap();
        assert_eq!(sum, TOTAL * (TOTAL - 1) / 2);
    }
}
