//! Runs a component in a native window with a WebView.
//!
//! The page is served from the custom `weft` protocol. View events arrive
//! over the WebView IPC channel; patches are sent back with
//! `evaluate_script`. Flushes and async tasks are user events of the tao
//! event loop, so everything runs on the UI thread.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::task::{Context, Wake, Waker};

use serde_json::Value;
use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tao::window::{Window, WindowBuilder};
use wry::http::{Response, StatusCode, header};
use wry::{WebView, WebViewBuilder};

use crate::host::{App, Platform, Session};

/// Page URL of the custom protocol; Windows and Android serve custom
/// protocols over `http://<scheme>.localhost`.
#[cfg(any(target_os = "windows", target_os = "android"))]
const START_URL: &str = "http://weft.localhost/";
#[cfg(not(any(target_os = "windows", target_os = "android")))]
const START_URL: &str = "weft://localhost/";

#[derive(Debug)]
enum UserEvent {
    /// A JSON message posted by the page.
    Message(String),
    Flush,
    /// Poll the task with this id.
    Poll(u64),
}

type Task = Pin<Box<dyn Future<Output = ()>>>;

/// [`Platform`] backed by the tao event loop.
struct Runner {
    proxy: EventLoopProxy<UserEvent>,
    flush_pending: Cell<bool>,
    tasks: RefCell<HashMap<u64, Task>>,
    next_task: Cell<u64>,
}

impl Platform for Runner {
    fn schedule_flush(&self) {
        if !self.flush_pending.replace(true) {
            // Sending only fails once the loop has exited.
            let _ = self.proxy.send_event(UserEvent::Flush);
        }
    }

    fn spawn(&self, task: Task) {
        let id = self.next_task.get();
        self.next_task.set(id + 1);
        self.tasks.borrow_mut().insert(id, task);
        // Polled from the loop rather than here, so the first poll never runs
        // inside event dispatch.
        let _ = self.proxy.send_event(UserEvent::Poll(id));
    }
}

impl Runner {
    fn poll(&self, id: u64) {
        // Removed while polling, so a task that wakes itself or touches other
        // tasks never meets a borrowed map.
        let Some(mut task) = self.tasks.borrow_mut().remove(&id) else {
            return;
        };
        let waker = Waker::from(Arc::new(TaskWaker {
            id,
            proxy: Mutex::new(self.proxy.clone()),
        }));
        if task
            .as_mut()
            .poll(&mut Context::from_waker(&waker))
            .is_pending()
        {
            self.tasks.borrow_mut().insert(id, task);
        }
    }
}

/// Wakes a task by posting a [`UserEvent::Poll`]; usable from any thread.
struct TaskWaker {
    id: u64,
    proxy: Mutex<EventLoopProxy<UserEvent>>,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        let proxy = self.proxy.lock().expect("waker proxy lock poisoned");
        let _ = proxy.send_event(UserEvent::Poll(self.id));
    }
}

/// Opens a window showing `app` and runs the UI event loop until the window
/// is closed, then exits the process.
///
/// Must be called on the main thread. Event handlers, async handlers, and
/// flushes all run on this thread.
///
/// # Panics
///
/// Panics when the window or the WebView cannot be created.
pub fn run(app: impl Into<App>) -> ! {
    let session = app.into().session;
    let view = session.view();
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let window = WindowBuilder::new()
        .with_title(view.title)
        .with_inner_size(LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .expect("weft: cannot create the window");

    let index_html = view.index_html();
    let ipc_proxy = proxy.clone();
    let builder = WebViewBuilder::new()
        .with_custom_protocol("weft".into(), move |_, request| {
            match view.asset(request.uri().path(), &index_html) {
                Some(asset) => Response::builder()
                    .header(header::CONTENT_TYPE, asset.mime)
                    .body(asset.body),
                None => Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Vec::new().into()),
            }
            .expect("static response parts are valid")
        })
        .with_ipc_handler(move |request| {
            let _ = ipc_proxy.send_event(UserEvent::Message(request.into_body()));
        })
        .with_url(START_URL);
    let webview = build_webview(builder, &window).expect("weft: cannot create the WebView");

    let runner = Rc::new(Runner {
        proxy,
        flush_pending: Cell::new(false),
        tasks: RefCell::new(HashMap::new()),
        next_task: Cell::new(0),
    });
    // The page cannot apply patches before it has started the component.
    let mut ready = false;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(UserEvent::Message(text)) => {
                handle_message(&*session, &runner, &webview, &mut ready, &text);
            }
            Event::UserEvent(UserEvent::Flush) => {
                runner.flush_pending.set(false);
                if ready && let Some(message) = session.flush() {
                    eval(&webview, &format!("__weft.apply({message})"));
                }
            }
            Event::UserEvent(UserEvent::Poll(id)) => runner.poll(id),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    })
}

fn handle_message(
    session: &dyn Session,
    runner: &Rc<Runner>,
    webview: &WebView,
    ready: &mut bool,
    text: &str,
) {
    let message: Value = match serde_json::from_str(text) {
        Ok(message) => message,
        Err(error) => {
            eprintln!("weft: ignoring malformed view message: {error}");
            return;
        }
    };
    match message["type"].as_str() {
        // Sent on every page load, including reloads.
        Some("ready") => {
            let platform: Rc<dyn Platform> = Rc::clone(runner) as Rc<dyn Platform>;
            let start = session.start(platform);
            *ready = true;
            eval(webview, &format!("__weft.start({start})"));
        }
        Some("event") => {
            let name = message["name"].as_str().unwrap_or_default();
            if let Err(error) = session.dispatch(name, &message["payload"]) {
                eprintln!("weft: {error}");
            }
        }
        _ => eprintln!("weft: ignoring unknown view message: {text}"),
    }
}

fn eval(webview: &WebView, script: &str) {
    if let Err(error) = webview.evaluate_script(script) {
        eprintln!("weft: cannot update the view: {error}");
    }
}

#[cfg(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "ios",
    target_os = "android"
))]
fn build_webview(builder: WebViewBuilder<'_>, window: &Window) -> wry::Result<WebView> {
    builder.build(window)
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "ios",
    target_os = "android"
)))]
fn build_webview(builder: WebViewBuilder<'_>, window: &Window) -> wry::Result<WebView> {
    use tao::platform::unix::WindowExtUnix;
    use wry::WebViewBuilderExtUnix;
    let vbox = window
        .default_vbox()
        .expect("tao windows on Linux have a default vbox");
    builder.build_gtk(vbox)
}
