//! Headless driver for testing generated components without a window.
//!
//! [`Driver`] plays the part of the WebView: it starts the component, sends
//! view events, collects the patch messages a flush would send, and runs the
//! futures of async handlers.

use std::cell::{Cell, RefCell};
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Wake, Waker};

use serde_json::Value;

use crate::host::{App, Platform, Session};

/// Runs a component like the WebView runner, driven step by step.
///
/// Messages have the shapes of the view protocol: [`start`](Self::start)
/// returns `{ "args": [...] }` and [`flush`](Self::flush) returns
/// `{ "patches": [[path, value], ...], "acks": n }`.
pub struct Driver {
    session: Box<dyn Session>,
    platform: Rc<TestPlatform>,
}

impl std::fmt::Debug for Driver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Driver")
            .field("component", &self.session.view().title)
            .finish_non_exhaustive()
    }
}

impl Driver {
    #[must_use]
    pub fn new(app: impl Into<App>) -> Self {
        Self {
            session: app.into().session,
            platform: Rc::new(TestPlatform::default()),
        }
    }

    /// Starts the component as a freshly loaded page does.
    pub fn start(&mut self) -> Value {
        let platform: Rc<dyn Platform> = Rc::clone(&self.platform) as Rc<dyn Platform>;
        self.session.start(platform)
    }

    /// Sends a view event; synchronous handlers run before this returns.
    ///
    /// # Panics
    ///
    /// Panics when `payload` does not decode as the event.
    pub fn send(&mut self, event: &str, payload: Value) {
        if let Err(error) = self.session.dispatch(event, &payload) {
            panic!("{error}");
        }
    }

    /// Whether a flush was requested since the last [`flush`](Self::flush).
    #[must_use]
    pub fn flush_requested(&self) -> bool {
        self.platform.flush_requested.get()
    }

    /// The message the next flush sends, or `None` when nothing changed.
    pub fn flush(&mut self) -> Option<Value> {
        self.platform.flush_requested.set(false);
        self.session.flush()
    }

    /// Polls async handler futures until none of them is woken.
    pub fn run_until_stalled(&mut self) {
        loop {
            let tasks = mem::take(&mut *self.platform.tasks.borrow_mut());
            let mut progressed = false;
            let mut pending = Vec::with_capacity(tasks.len());
            for (flag, mut task) in tasks {
                if !flag.0.swap(false, Ordering::SeqCst) {
                    pending.push((flag, task));
                    continue;
                }
                progressed = true;
                let waker = Waker::from(Arc::clone(&flag));
                if task
                    .as_mut()
                    .poll(&mut Context::from_waker(&waker))
                    .is_pending()
                {
                    pending.push((flag, task));
                }
            }
            self.platform.tasks.borrow_mut().extend(pending);
            if !progressed {
                return;
            }
        }
    }
}

type Task = Pin<Box<dyn Future<Output = ()>>>;

/// Set when a task is woken.
struct WakeFlag(AtomicBool);

impl Wake for WakeFlag {
    fn wake(self: Arc<Self>) {
        self.0.store(true, Ordering::SeqCst);
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.store(true, Ordering::SeqCst);
    }
}

#[derive(Default)]
struct TestPlatform {
    flush_requested: Cell<bool>,
    tasks: RefCell<Vec<(Arc<WakeFlag>, Task)>>,
}

impl Platform for TestPlatform {
    fn schedule_flush(&self) {
        self.flush_requested.set(true);
    }

    fn spawn(&self, task: Task) {
        let flag = Arc::new(WakeFlag(AtomicBool::new(true)));
        self.tasks.borrow_mut().push((flag, task));
    }
}
