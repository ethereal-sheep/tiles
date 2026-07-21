use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::marker::PhantomData;

// --- SignalId ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalId(u64);

impl SignalId {
    fn new(widget_path: u64, local_index: usize) -> Self {
        let mut h = widget_path;
        h = h
            .wrapping_mul(6364136223846793005)
            .wrapping_add(local_index as u64);
        Self(h)
    }
}

// --- Signal<T> ---

pub struct Signal<T: 'static> {
    id: SignalId,
    _marker: PhantomData<fn() -> T>,
}

impl<T: 'static> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for Signal<T> {}

impl<T: Clone + 'static> Signal<T> {
    pub fn get(self) -> T {
        with_runtime(|rt| {
            let storage = rt.storage.borrow();
            storage
                .get(&self.id)
                .and_then(|b| b.downcast_ref::<T>())
                .expect("signal: type mismatch or not initialized")
                .clone()
        })
    }

    pub fn set(self, value: T) {
        with_runtime(|rt| {
            let mut storage = rt.storage.borrow_mut();
            if let Some(slot) = storage.get_mut(&self.id) {
                *slot.downcast_mut::<T>().expect("signal: type mismatch") = value;
            }
        })
    }
}

// --- SignalRuntime ---

pub struct SignalRuntime {
    storage: RefCell<HashMap<SignalId, Box<dyn Any>>>,
    widget_stack: RefCell<Vec<u64>>,
    local_counter: Cell<usize>,
}

impl SignalRuntime {
    pub fn new() -> Self {
        Self {
            storage: RefCell::new(HashMap::new()),
            widget_stack: RefCell::new(Vec::new()),
            local_counter: Cell::new(0),
        }
    }
}

// --- Thread-local runtime ---

thread_local! {
    static RUNTIME: Cell<*const SignalRuntime> = const { Cell::new(std::ptr::null()) };
}

pub(crate) fn set_runtime(rt: &SignalRuntime) {
    RUNTIME.with(|r| r.set(rt as *const _));
}

pub(crate) fn clear_runtime() {
    RUNTIME.with(|r| r.set(std::ptr::null()));
}

fn with_runtime<F, R>(f: F) -> R
where
    F: FnOnce(&SignalRuntime) -> R,
{
    RUNTIME.with(|r| {
        let ptr = r.get();
        assert!(
            !ptr.is_null(),
            "signal: no runtime active (called outside ui/handler)"
        );
        f(unsafe { &*ptr })
    })
}

fn try_with_runtime<F>(f: F) -> bool
where
    F: FnOnce(&SignalRuntime),
{
    RUNTIME.with(|r| {
        let ptr = r.get();
        if ptr.is_null() {
            return false;
        }
        f(unsafe { &*ptr });
        true
    })
}

// --- Public API ---

pub fn signal<T: Clone + 'static>(default: T) -> Signal<T> {
    with_runtime(|rt| {
        let widget_path = {
            let stack = rt.widget_stack.borrow();
            *stack
                .last()
                .expect("signal: no widget context (missing #[widget_fn]?)")
        };
        let local_index = rt.local_counter.get();
        rt.local_counter.set(local_index + 1);

        let id = SignalId::new(widget_path, local_index);

        let mut storage = rt.storage.borrow_mut();
        if !storage.contains_key(&id) {
            storage.insert(id, Box::new(default));
        }

        Signal {
            id,
            _marker: PhantomData,
        }
    })
}

// --- Widget path management (called by macro-generated code) ---

#[doc(hidden)]
pub fn __push_widget(path: u64) {
    if !try_with_runtime(|rt| {
        let parent = rt.widget_stack.borrow().last().copied().unwrap_or(0);
        let combined = parent.wrapping_mul(2654435761).wrapping_add(path);
        rt.widget_stack.borrow_mut().push(combined);
        rt.local_counter.set(0);
    }) {
        // No runtime active — no-op (e.g. in tests without signal setup)
    }
}

#[doc(hidden)]
pub fn __pop_widget() {
    try_with_runtime(|rt| {
        rt.widget_stack.borrow_mut().pop();
    });
}

#[doc(hidden)]
pub fn __widget_id(file: &str, line: u32, col: u32) -> u64 {
    let mut h: u64 = 14695981039346656037;
    for b in file.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    h ^= line as u64;
    h = h.wrapping_mul(1099511628211);
    h ^= col as u64;
    h = h.wrapping_mul(1099511628211);
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_test_runtime(f: impl FnOnce()) {
        let rt = SignalRuntime::new();
        set_runtime(&rt);
        f();
        clear_runtime();
    }

    #[test]
    fn signal_create_and_read_default() {
        with_test_runtime(|| {
            __push_widget(1);
            let s = signal(42i32);
            assert_eq!(s.get(), 42);
            __pop_widget();
        });
    }

    #[test]
    fn signal_set_and_get() {
        with_test_runtime(|| {
            __push_widget(1);
            let s = signal(0i32);
            s.set(99);
            assert_eq!(s.get(), 99);
            __pop_widget();
        });
    }

    #[test]
    fn signal_persists_across_frames() {
        let rt = SignalRuntime::new();

        // Frame 1: create and set
        set_runtime(&rt);
        __push_widget(1);
        let s = signal(0i32);
        s.set(5);
        __pop_widget();
        clear_runtime();

        // Frame 2: same widget path → same signal, retains value
        set_runtime(&rt);
        __push_widget(1);
        let s2 = signal(0i32); // default ignored since already exists
        assert_eq!(s2.get(), 5);
        __pop_widget();
        clear_runtime();
    }

    #[test]
    fn multiple_signals_in_one_widget() {
        with_test_runtime(|| {
            __push_widget(1);
            let a = signal(10i32);
            let b = signal(20i32);
            let c = signal(30i32);
            assert_eq!(a.get(), 10);
            assert_eq!(b.get(), 20);
            assert_eq!(c.get(), 30);
            a.set(100);
            assert_eq!(a.get(), 100);
            assert_eq!(b.get(), 20);
            __pop_widget();
        });
    }

    #[test]
    fn different_widget_paths_are_independent() {
        with_test_runtime(|| {
            // Widget at path 1
            __push_widget(1);
            let s1 = signal(0i32);
            s1.set(111);
            __pop_widget();

            // Widget at path 2
            __push_widget(2);
            let s2 = signal(0i32);
            s2.set(222);
            __pop_widget();

            // Verify independence
            __push_widget(1);
            let s1_again = signal(0i32);
            assert_eq!(s1_again.get(), 111);
            __pop_widget();

            __push_widget(2);
            let s2_again = signal(0i32);
            assert_eq!(s2_again.get(), 222);
            __pop_widget();
        });
    }

    #[test]
    fn nested_widget_paths_are_independent() {
        with_test_runtime(|| {
            __push_widget(1);
            let outer = signal(100i32);

            // Nested child widget
            __push_widget(2);
            let inner = signal(200i32);
            assert_eq!(inner.get(), 200);
            __pop_widget();

            assert_eq!(outer.get(), 100);
            __pop_widget();
        });
    }

    #[test]
    fn same_path_different_parent_are_independent() {
        with_test_runtime(|| {
            // Parent A → child with path 99
            __push_widget(10);
            __push_widget(99);
            let s1 = signal(1i32);
            __pop_widget();
            __pop_widget();

            // Parent B → child with path 99
            __push_widget(20);
            __push_widget(99);
            let s2 = signal(2i32);
            __pop_widget();
            __pop_widget();

            assert_ne!(s1.id, s2.id);
        });
    }

    #[test]
    fn local_counter_resets_on_push() {
        with_test_runtime(|| {
            __push_widget(1);
            let _a = signal(0i32); // index 0
            let _b = signal(0i32); // index 1
            __pop_widget();

            // Push same widget again (simulates next frame)
            __push_widget(1);
            let c = signal(0i32); // index 0 again — same SignalId as _a
            let d = signal(0i32); // index 1 again — same SignalId as _b
            // They should get the same IDs
            assert_eq!(c.id, _a.id);
            assert_eq!(d.id, _b.id);
            __pop_widget();
        });
    }

    #[test]
    fn signal_works_with_string() {
        with_test_runtime(|| {
            __push_widget(1);
            let s = signal("hello".to_string());
            assert_eq!(s.get(), "hello");
            s.set("world".to_string());
            assert_eq!(s.get(), "world");
            __pop_widget();
        });
    }

    #[test]
    fn signal_works_with_vec() {
        with_test_runtime(|| {
            __push_widget(1);
            let s = signal(vec![1, 2, 3]);
            assert_eq!(s.get(), vec![1, 2, 3]);
            s.set(vec![4, 5]);
            assert_eq!(s.get(), vec![4, 5]);
            __pop_widget();
        });
    }

    #[test]
    fn signal_is_copy() {
        with_test_runtime(|| {
            __push_widget(1);
            let s = signal(0i32);
            let s2 = s; // Copy
            let s3 = s; // Copy again
            s2.set(10);
            assert_eq!(s3.get(), 10);
            assert_eq!(s.get(), 10);
            __pop_widget();
        });
    }

    #[test]
    fn widget_id_produces_different_hashes() {
        let id1 = __widget_id("file.rs", 10, 5);
        let id2 = __widget_id("file.rs", 10, 6);
        let id3 = __widget_id("file.rs", 11, 5);
        let id4 = __widget_id("other.rs", 10, 5);
        assert_ne!(id1, id2);
        assert_ne!(id1, id3);
        assert_ne!(id1, id4);
        assert_ne!(id2, id3);
    }

    #[test]
    fn push_widget_differentiates_loop_iterations() {
        with_test_runtime(|| {
            __push_widget(0); // parent widget
            // Loop iteration 0
            let s0 = signal(0i32);
            s0.set(100);
            __pop_widget();

            // Loop iteration 1
            __push_widget(1);
            let s1 = signal(0i32);
            s1.set(200);
            __pop_widget();

            // Verify they're independent
            __push_widget(0);
            let s0_again = signal(0i32);
            assert_eq!(s0_again.get(), 100);
            __pop_widget();

            __push_widget(1);
            let s1_again = signal(0i32);
            assert_eq!(s1_again.get(), 200);
            __pop_widget();
        });
    }

    #[test]
    #[should_panic(expected = "signal: no widget context")]
    fn create_signal_without_widget_context_panics() {
        with_test_runtime(|| {
            // No __push_widget — should panic
            let _s = signal(0i32);
        });
    }

    #[test]
    #[should_panic(expected = "signal: no runtime active")]
    fn signal_get_without_runtime_panics() {
        // Create signal in valid context, then try to read outside
        let rt = SignalRuntime::new();
        set_runtime(&rt);
        __push_widget(1);
        let s = signal(42i32);
        __pop_widget();
        clear_runtime();

        // Now try to read without runtime — should panic
        let _ = s.get();
    }
}
