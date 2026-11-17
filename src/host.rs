//! Host-side component state: the model, change tracking, event dispatch,
//! and the patches sent to the view.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, HashMap};
use std::fmt;
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::rc::Rc;

use serde_json::{Value, json};

use crate::json::{DecodeError, FromJson, ToJson};
use crate::view::View;

/// A field path below the handler model, e.g. `["user", "name"]`.
type Path = Vec<&'static str>;

/// A model declared in a `.weft` file; implemented by generated code.
pub trait Model: ToJson + FromJson + 'static {
    /// Change-tracking accessor handed to event handlers.
    type Mut<'a>;

    /// Wraps `self` so that changes are recorded in `changes`.
    fn track<'a>(&'a mut self, changes: Changes<'a>) -> Self::Mut<'a>;

    /// JSON of the value at `path`, a field path recorded by [`Changes`].
    fn path_json(&self, path: &[&'static str]) -> Value;
}

/// Components whose first parameter is not a model have nothing to change.
impl Model for () {
    type Mut<'a> = ();

    fn track<'a>(&'a mut self, _: Changes<'a>) {}

    fn path_json(&self, path: &[&'static str]) -> Value {
        crate::json::unknown_path("()", path)
    }
}

/// Records which fields of a model changed; used by generated accessors.
#[derive(Debug)]
pub struct Changes<'a> {
    paths: &'a mut BTreeSet<Path>,
    prefix: Path,
}

impl<'a> Changes<'a> {
    fn root(paths: &'a mut BTreeSet<Path>) -> Self {
        Self {
            paths,
            prefix: Vec::new(),
        }
    }

    /// Records a change of `field`.
    pub fn mark(&mut self, field: &'static str) {
        let mut path = self.prefix.clone();
        path.push(field);
        self.paths.insert(path);
    }

    /// Changes below the nested model in `field`.
    pub fn field(&mut self, field: &'static str) -> Changes<'_> {
        let mut prefix = self.prefix.clone();
        prefix.push(field);
        Changes {
            paths: &mut *self.paths,
            prefix,
        }
    }
}

/// Services of the event loop running a component.
pub(crate) trait Platform {
    /// Requests a call to [`Session::flush`] soon; repeated requests coalesce.
    fn schedule_flush(&self);
    /// Runs `task` to completion on the UI thread.
    fn spawn(&self, task: Pin<Box<dyn Future<Output = ()>>>);
}

/// State shared by the host and every [`ModelHandle`].
struct State<M> {
    model: RefCell<M>,
    changes: RefCell<BTreeSet<Path>>,
    /// Events dispatched since the last flush.
    acks: Cell<u32>,
    platform: RefCell<Option<Rc<dyn Platform>>>,
}

impl<M: Model> State<M> {
    fn update<R>(&self, f: impl FnOnce(&mut M::Mut<'_>) -> R) -> R {
        let result = {
            let mut model = self
                .model
                .try_borrow_mut()
                .expect("weft: the model is already being updated (nested `ModelHandle::update`?)");
            let mut changes = self.changes.borrow_mut();
            let mut tracked = model.track(Changes::root(&mut changes));
            f(&mut tracked)
        };
        self.request_flush();
        result
    }

    fn request_flush(&self) {
        if self.acks.get() == 0 && self.changes.borrow().is_empty() {
            return;
        }
        if let Some(platform) = &*self.platform.borrow() {
            platform.schedule_flush();
        }
    }

    fn spawn(&self, task: Pin<Box<dyn Future<Output = ()>>>) {
        self.platform
            .borrow()
            .as_ref()
            .expect("weft: events are only dispatched to a running component")
            .spawn(task);
    }
}

/// Shared access to the model for asynchronous event handlers.
///
/// Clones refer to the same model. The handle only lives on the UI thread;
/// borrows end with each [`update`](Self::update) call, so it can be held
/// across `.await` points.
pub struct ModelHandle<M> {
    state: Rc<State<M>>,
}

impl<M> Clone for ModelHandle<M> {
    fn clone(&self) -> Self {
        Self {
            state: Rc::clone(&self.state),
        }
    }
}

impl<M> fmt::Debug for ModelHandle<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ModelHandle").finish_non_exhaustive()
    }
}

impl<M: Model> ModelHandle<M> {
    /// Changes the model through its change-tracking accessor; the view is
    /// patched at the next flush.
    ///
    /// # Panics
    ///
    /// Panics when called from inside another `update` of the same model.
    pub fn update<R>(&self, f: impl FnOnce(&mut M::Mut<'_>) -> R) -> R {
        self.state.update(f)
    }
}

type Handler<M> = Box<dyn FnMut(&Rc<State<M>>, &Value) -> Result<(), DecodeError>>;

/// Runtime of a generated root component.
pub struct Host<M> {
    view: &'static View,
    state: Rc<State<M>>,
    /// JSON of the constructor arguments the host never changes.
    rest: Vec<Value>,
    /// Whether the model is the first constructor argument.
    model_arg: bool,
    handlers: RefCell<HashMap<&'static str, Vec<Handler<M>>>>,
}

impl<M: Model> Host<M> {
    /// A component whose first argument is `model`, followed by `rest`.
    #[must_use]
    pub fn new(view: &'static View, model: M, rest: Vec<Value>) -> Self {
        Self::with_args(view, model, rest, true)
    }

    fn with_args(view: &'static View, model: M, rest: Vec<Value>, model_arg: bool) -> Self {
        Self {
            view,
            state: Rc::new(State {
                model: RefCell::new(model),
                changes: RefCell::new(BTreeSet::new()),
                acks: Cell::new(0),
                platform: RefCell::new(None),
            }),
            rest,
            model_arg,
            handlers: RefCell::new(HashMap::new()),
        }
    }

    fn add(&self, event: &'static str, handler: Handler<M>) {
        self.handlers
            .borrow_mut()
            .entry(event)
            .or_default()
            .push(handler);
    }

    /// Registers a synchronous handler for `event`.
    pub fn on<E, F>(&self, event: &'static str, mut handler: F)
    where
        E: FromJson + 'static,
        F: FnMut(&mut M::Mut<'_>, E) + 'static,
    {
        self.add(
            event,
            Box::new(move |state, payload| {
                let event = E::from_json(payload)?;
                state.update(|model| handler(model, event));
                Ok(())
            }),
        );
    }

    /// Registers a handler for `event` whose future runs on the UI thread.
    pub fn on_async<E, F, Fut>(&self, event: &'static str, mut handler: F)
    where
        E: FromJson + 'static,
        F: FnMut(ModelHandle<M>, E) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        self.add(
            event,
            Box::new(move |state, payload| {
                let event = E::from_json(payload)?;
                let handle = ModelHandle {
                    state: Rc::clone(state),
                };
                state.spawn(Box::pin(handler(handle, event)));
                Ok(())
            }),
        );
    }

    /// Type-erases the component for [`crate::run`].
    #[must_use]
    pub fn into_app(self) -> App {
        App {
            session: Box::new(self),
        }
    }
}

impl Host<()> {
    /// A component without a handler model; `args` are all its arguments.
    #[must_use]
    pub fn fixed(view: &'static View, args: Vec<Value>) -> Self {
        Self::with_args(view, (), args, false)
    }
}

/// A view event could not be decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DispatchError {
    event: String,
    error: DecodeError,
}

impl fmt::Display for DispatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid payload for event `{}`: {}",
            self.event, self.error
        )
    }
}

impl std::error::Error for DispatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Object-safe interface between a component and the loop that runs it.
///
/// Messages follow the view protocol of `@weft-ui/runtime/remote`:
/// `start` yields `{ args }`, `flush` yields `{ patches, acks }`.
pub(crate) trait Session {
    fn view(&self) -> &'static View;
    /// Attaches the platform and returns the full state; pending changes are
    /// dropped because the snapshot already contains them.
    fn start(&self, platform: Rc<dyn Platform>) -> Value;
    /// Runs the handlers of `event`; the event is acknowledged at the next
    /// flush even when its payload is invalid.
    fn dispatch(&self, event: &str, payload: &Value) -> Result<(), DispatchError>;
    /// Patches for the changes since the last flush, or `None` when there is
    /// nothing to send.
    fn flush(&self) -> Option<Value>;
}

impl<M: Model> Session for Host<M> {
    fn view(&self) -> &'static View {
        self.view
    }

    fn start(&self, platform: Rc<dyn Platform>) -> Value {
        *self.state.platform.borrow_mut() = Some(platform);
        self.state.changes.borrow_mut().clear();
        self.state.acks.set(0);
        let mut args = Vec::with_capacity(self.rest.len() + 1);
        if self.model_arg {
            args.push(self.state.model.borrow().to_json());
        }
        args.extend(self.rest.iter().cloned());
        json!({ "args": args })
    }

    fn dispatch(&self, event: &str, payload: &Value) -> Result<(), DispatchError> {
        self.state.acks.set(self.state.acks.get() + 1);
        let mut result = Ok(());
        if let Some(handlers) = self.handlers.borrow_mut().get_mut(event) {
            for handler in handlers {
                if let Err(error) = handler(&self.state, payload) {
                    result = Err(DispatchError {
                        event: event.to_owned(),
                        error,
                    });
                }
            }
        }
        self.state.request_flush();
        result
    }

    fn flush(&self) -> Option<Value> {
        let paths = mem::take(&mut *self.state.changes.borrow_mut());
        let acks = self.state.acks.replace(0);
        if paths.is_empty() && acks == 0 {
            return None;
        }
        let model = self.state.model.borrow();
        let mut patches = Vec::with_capacity(paths.len());
        // Sorted order puts a path right before the paths it prefixes, so
        // comparing with the last kept path drops every covered change.
        let mut kept: Option<&Path> = None;
        for path in &paths {
            if kept.is_some_and(|prefix| path.starts_with(prefix)) {
                continue;
            }
            patches.push(json!([path, model.path_json(path)]));
            kept = Some(path);
        }
        Some(json!({ "patches": patches, "acks": acks }))
    }
}

/// A root component ready to [`run`](crate::run).
///
/// Generated component types convert into it with `From`.
pub struct App {
    pub(crate) session: Box<dyn Session>,
}

impl fmt::Debug for App {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("App")
            .field("component", &self.session.view().title)
            .finish_non_exhaustive()
    }
}
