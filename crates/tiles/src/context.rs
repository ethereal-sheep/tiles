use std::any::TypeId;
use std::cell::Cell;
use std::marker::PhantomData;

use crate::runner::State;

// --- Phase gate ---
//
// Building spans the App::ui() call (tree construction); Evaluating spans
// resolved.evaluate() (hit-testing + handler dispatch). Both windows are set
// by the runner. Outside either (Idle), App/State are only reachable via the
// lifecycle methods' own &mut parameters, never through get_app/get_state.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase {
    Idle,
    Building,
    Evaluating,
}

thread_local! {
    static PHASE: Cell<Phase> = const { Cell::new(Phase::Idle) };
}

pub(crate) fn set_phase(phase: Phase) {
    PHASE.with(|p| p.set(phase));
}

pub(crate) fn phase() -> Phase {
    PHASE.with(|p| p.get())
}

// --- Storage ---
//
// `thread_local!` statics can't be generic over an outer type parameter (the
// macro expands to a top-level item, which Rust doesn't monomorphize per
// caller) — so each registered value needs its own concrete thread-local.
// App is generic over the caller's app type, so its slot is type-erased and
// TypeId-checked; State is already concrete, so its slot holds a typed
// pointer directly. The `with`/`with_mut` mechanics (phase check, reentrancy
// guard) are shared via `access` below rather than duplicated per slot.

thread_local! {
    static APP_PTR: Cell<*mut ()> = const { Cell::new(std::ptr::null_mut()) };
    static APP_TYPE: Cell<Option<TypeId>> = const { Cell::new(None) };
    static APP_BORROWED: Cell<bool> = const { Cell::new(false) };
    static STATE_PTR: Cell<*mut State> = const { Cell::new(std::ptr::null_mut()) };
    static STATE_BORROWED: Cell<bool> = const { Cell::new(false) };
}

/// Registered once at `Runner` construction — the App instance never moves
/// for the process lifetime, so no per-frame re-registration is needed.
pub(crate) fn register_app<A: 'static>(app: *mut A) {
    APP_PTR.with(|p| p.set(app as *mut ()));
    APP_TYPE.with(|t| t.set(Some(TypeId::of::<A>())));
}

/// Registered once at `Runner` construction, mirroring `register_app` — the
/// runner's `State` field never moves for the process lifetime either.
pub(crate) fn register_state(state: *mut State) {
    STATE_PTR.with(|p| p.set(state));
}

/// Shared `with`/`with_mut` mechanics: phase gate + reentrancy guard around a
/// raw-pointer dereference. `name` is the public constructor name
/// (`"get_app"`/`"get_state"`), used only so panic messages point at the
/// actual call site.
fn access<T, R>(
    name: &'static str,
    ptr: *mut T,
    borrowed: &'static std::thread::LocalKey<Cell<bool>>,
    require_evaluating: bool,
    f: impl FnOnce(*mut T) -> R,
) -> R {
    assert!(
        !ptr.is_null(),
        "{name}(): not registered (wrong type, or called outside the run loop?)"
    );
    if require_evaluating {
        assert!(
            phase() == Phase::Evaluating,
            "{name}().with_mut() is only allowed inside event handlers, not during ui() build"
        );
    } else {
        assert!(
            phase() != Phase::Idle,
            "{name}() is only available during ui() build or event handlers — already \
             reachable via &mut in init/update/draw/on_key/on_mouse"
        );
    }
    assert!(
        !borrowed.with(|b| b.replace(true)),
        "{name}(): reentrant borrow — with()/with_mut() called while another borrow is active"
    );
    let result = f(ptr);
    borrowed.with(|b| b.set(false));
    result
}

// --- AppContext<A> ---

pub struct AppContext<A: 'static> {
    _marker: PhantomData<fn() -> A>,
}

impl<A: 'static> Clone for AppContext<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: 'static> Copy for AppContext<A> {}

impl<A: 'static> AppContext<A> {
    pub fn with<R>(self, f: impl FnOnce(&A) -> R) -> R {
        access("get_app", self.ptr(), &APP_BORROWED, false, |p| f(unsafe {
            &*p
        }))
    }

    pub fn with_mut<R>(self, f: impl FnOnce(&mut A) -> R) -> R {
        access("get_app", self.ptr(), &APP_BORROWED, true, |p| f(unsafe {
            &mut *p
        }))
    }

    fn ptr(self) -> *mut A {
        APP_PTR.with(|p| p.get()) as *mut A
    }
}

/// Returns a copyable handle to the running `App` instance. Reads (`.with`)
/// are allowed during both tree-building and event handling; mutation
/// (`.with_mut`) is only allowed inside event handlers.
pub fn get_app<A: 'static>() -> AppContext<A> {
    let registered = APP_TYPE.with(|t| t.get());
    match registered {
        Some(id) if id == TypeId::of::<A>() => {}
        Some(_) => panic!(
            "get_app::<{}>() called but a different App type is registered",
            std::any::type_name::<A>()
        ),
        None => panic!("get_app(): no App registered (called outside the run loop?)"),
    }
    AppContext {
        _marker: PhantomData,
    }
}

// --- StateContext ---

#[derive(Clone, Copy)]
pub struct StateContext {
    _private: (),
}

impl StateContext {
    pub fn with<R>(self, f: impl FnOnce(&State) -> R) -> R {
        access("get_state", self.ptr(), &STATE_BORROWED, false, |p| f(unsafe {
            &*p
        }))
    }

    pub fn with_mut<R>(self, f: impl FnOnce(&mut State) -> R) -> R {
        access("get_state", self.ptr(), &STATE_BORROWED, true, |p| f(unsafe {
            &mut *p
        }))
    }

    fn ptr(self) -> *mut State {
        STATE_PTR.with(|p| p.get())
    }
}

/// Returns a copyable handle to the running `State`. Reads (`.with`) are
/// allowed during both tree-building and event handling; mutation
/// (`.with_mut`) is only allowed inside event handlers.
pub fn get_state() -> StateContext {
    StateContext { _private: () }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Demo {
        count: i32,
    }

    struct Other;

    fn with_test_app<R>(app: &mut Demo, f: impl FnOnce() -> R) -> R {
        register_app(app as *mut Demo);
        let r = f();
        APP_PTR.with(|p| p.set(std::ptr::null_mut()));
        APP_TYPE.with(|t| t.set(None));
        r
    }

    fn with_test_state<R>(state: &mut State, f: impl FnOnce() -> R) -> R {
        register_state(state as *mut State);
        let r = f();
        STATE_PTR.with(|p| p.set(std::ptr::null_mut()));
        r
    }

    #[test]
    fn app_with_reads_value() {
        let mut app = Demo { count: 5 };
        with_test_app(&mut app, || {
            set_phase(Phase::Building);
            let v = get_app::<Demo>().with(|a| a.count);
            set_phase(Phase::Idle);
            assert_eq!(v, 5);
        });
    }

    #[test]
    fn app_with_mut_during_evaluating_succeeds() {
        let mut app = Demo { count: 0 };
        with_test_app(&mut app, || {
            set_phase(Phase::Evaluating);
            get_app::<Demo>().with_mut(|a| a.count += 1);
            set_phase(Phase::Idle);
        });
        assert_eq!(app.count, 1);
    }

    #[test]
    #[should_panic(expected = "only allowed inside event handlers")]
    fn app_with_mut_during_building_panics() {
        let mut app = Demo { count: 0 };
        with_test_app(&mut app, || {
            set_phase(Phase::Building);
            get_app::<Demo>().with_mut(|a| a.count += 1);
        });
    }

    #[test]
    #[should_panic(expected = "different App type is registered")]
    fn app_wrong_type_panics_immediately() {
        let mut app = Demo { count: 0 };
        with_test_app(&mut app, || {
            let _ = get_app::<Other>();
        });
    }

    #[test]
    #[should_panic(expected = "reentrant borrow")]
    fn app_reentrant_with_mut_panics() {
        let mut app = Demo { count: 0 };
        with_test_app(&mut app, || {
            set_phase(Phase::Evaluating);
            get_app::<Demo>().with_mut(|_| {
                get_app::<Demo>().with_mut(|_| {});
            });
        });
    }

    #[test]
    fn state_with_mut_during_evaluating_succeeds() {
        let mut state = State::new_for_test(256, 256);
        with_test_state(&mut state, || {
            set_phase(Phase::Evaluating);
            get_state().with_mut(|s| s.set_debug(true));
            set_phase(Phase::Idle);
        });
        assert!(state.is_debug());
    }

    #[test]
    #[should_panic(expected = "only allowed inside event handlers")]
    fn state_with_mut_during_building_panics() {
        let mut state = State::new_for_test(256, 256);
        with_test_state(&mut state, || {
            set_phase(Phase::Building);
            get_state().with_mut(|s| s.set_debug(true));
        });
    }
}
