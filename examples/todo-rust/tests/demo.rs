//! The demo app driven headlessly through the view protocol.

use std::cell::RefCell;
use std::future;
use std::rc::Rc;
use std::task::{Poll, Waker};

use serde_json::{Value, json};
use weft_ui::testing::Driver;

fn todos(ids: &[(i64, &str)]) -> Value {
    ids.iter()
        .map(|(id, name)| json!({ "id": id, "name": name }))
        .collect()
}

#[test]
fn start_sends_the_initial_model() {
    let mut driver = Driver::new(todo_rust::app());
    let start = driver.start();
    assert_eq!(
        start,
        json!({ "args": [{
            "draft": "",
            "todos": todos(&[
                (1, "Describe a view in Weft"),
                (2, "Expand the list"),
                (3, "Remove a finished task"),
            ]),
        }] })
    );
    assert_eq!(driver.flush(), None, "starting leaves nothing to flush");
}

#[test]
fn typing_patches_only_the_draft() {
    let mut driver = Driver::new(todo_rust::app());
    driver.start();
    driver.send("TextChanged", json!({ "value": "milk" }));
    assert!(driver.flush_requested());
    assert_eq!(
        driver.flush(),
        Some(json!({ "patches": [[["draft"], "milk"]], "acks": 1 }))
    );
    assert_eq!(driver.flush(), None);
}

#[test]
fn unchanged_values_are_acknowledged_without_patches() {
    let mut driver = Driver::new(todo_rust::app());
    driver.start();
    driver.send("TextChanged", json!({ "value": "" }));
    assert_eq!(driver.flush(), Some(json!({ "patches": [], "acks": 1 })));
}

#[test]
fn events_between_flushes_are_batched() {
    let mut driver = Driver::new(todo_rust::app());
    driver.start();
    driver.send("Remove", json!({ "id": 2 }));
    driver.send("TextChanged", json!({ "value": "a" }));
    driver.send("TextChanged", json!({ "value": "ab" }));
    assert_eq!(
        driver.flush(),
        Some(json!({
            "patches": [
                [["draft"], "ab"],
                [["todos"], todos(&[(1, "Describe a view in Weft"), (3, "Remove a finished task")])],
            ],
            "acks": 3,
        }))
    );
}

#[test]
#[should_panic(expected = "invalid payload for event `Remove`: Remove.id: expected an integer")]
fn invalid_payloads_are_reported() {
    let mut driver = Driver::new(todo_rust::app());
    driver.start();
    driver.send("Remove", json!({ "id": "two" }));
}

/// A future completed from the test, standing in for real async work.
#[derive(Default)]
struct Gate {
    open: bool,
    waker: Option<Waker>,
}

#[test]
fn async_handlers_patch_after_await() {
    let app = todo_rust::app();
    let gate = Rc::new(RefCell::new(Gate::default()));
    let task_gate = Rc::clone(&gate);
    app.on_remove_async(move |model, e| {
        let gate = Rc::clone(&task_gate);
        async move {
            model.update(|m| m.set_draft(format!("removing {}", e.id)));
            future::poll_fn(|cx| {
                let mut gate = gate.borrow_mut();
                if gate.open {
                    Poll::Ready(())
                } else {
                    gate.waker = Some(cx.waker().clone());
                    Poll::Pending
                }
            })
            .await;
            model.update(|m| m.draft_mut().clear());
        }
    });
    let mut driver = Driver::new(app);
    driver.start();

    driver.send("Remove", json!({ "id": 1 }));
    let remaining = todos(&[(2, "Expand the list"), (3, "Remove a finished task")]);
    assert_eq!(
        driver.flush(),
        Some(json!({ "patches": [[["todos"], remaining]], "acks": 1 })),
        "the sync handler is flushed before the async one first runs"
    );

    driver.run_until_stalled();
    assert_eq!(
        driver.flush(),
        Some(json!({ "patches": [[["draft"], "removing 1"]], "acks": 0 }))
    );
    driver.run_until_stalled();
    assert_eq!(driver.flush(), None, "the task waits for the gate");

    let waker = {
        let mut gate = gate.borrow_mut();
        gate.open = true;
        gate.waker.take().expect("the task registered a waker")
    };
    waker.wake();
    driver.run_until_stalled();
    assert_eq!(
        driver.flush(),
        Some(json!({ "patches": [[["draft"], ""]], "acks": 0 }))
    );
}
