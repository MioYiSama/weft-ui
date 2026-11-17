//! Generated Rust host bindings for nested, optional, and keyword-named
//! fields, imported components, and components without a model.

use std::cell::RefCell;
use std::rc::Rc;

use serde_json::json;
use weft_ui::testing::Driver;
use weft_ui_host_tests::profile::{Address, Badge, Profile, ProfileView, User, events};

fn user(name: &str, city: &str) -> User {
    User {
        name: name.to_owned(),
        address: Address {
            city: city.to_owned(),
            zip: None,
        },
        r#type: 1,
        self_: false,
    }
}

fn profile() -> Profile {
    Profile {
        user: user("Ada", "London"),
        best: None,
        tags: vec!["math".to_owned()],
        score: 1.5,
        visits: 3,
    }
}

#[test]
fn start_uses_declared_field_names() {
    let mut driver = Driver::new(ProfileView::new(profile()));
    let user = json!({
        "name": "Ada",
        "address": { "city": "London", "zip": null },
        "type": 1,
        "self": false,
    });
    assert_eq!(
        driver.start(),
        json!({ "args": [{
            "user": user,
            "best": null,
            "tags": ["math"],
            "score": 1.5,
            "visits": 3,
        }] })
    );
}

#[test]
fn nested_changes_patch_their_own_path() {
    let app = ProfileView::new(profile());
    app.on_loaded(|profile, _: events::Loaded| {
        let mut user = profile.user_mut();
        user.address_mut().set_city("Paris".to_owned());
        user.set_type(2);
        *profile.visits_mut() += 1;
    });
    let mut driver = Driver::new(app);
    driver.start();
    driver.send("Loaded", json!({}));
    assert_eq!(
        driver.flush(),
        Some(json!({
            "patches": [
                [["user", "address", "city"], "Paris"],
                [["user", "type"], 2],
                [["visits"], 4],
            ],
            "acks": 1,
        }))
    );
}

#[test]
fn replacing_a_model_covers_changes_inside_it() {
    let app = ProfileView::new(profile());
    app.on_loaded(|profile, _: events::Loaded| {
        profile.user_mut().set_name("Grace".to_owned());
        profile.set_user(user("Linus", "Helsinki"));
        *profile.best_mut() = Some(user("Ada", "London"));
    });
    let mut driver = Driver::new(app);
    driver.start();
    driver.send("Loaded", json!({}));
    let message = driver.flush().expect("changes were made");
    let paths: Vec<_> = message["patches"]
        .as_array()
        .expect("patches are an array")
        .iter()
        .map(|patch| patch[0].clone())
        .collect();
    assert_eq!(paths, [json!(["best"]), json!(["user"])]);
    assert_eq!(message["patches"][1][1]["name"], "Linus");
}

#[test]
fn model_payloads_decode_into_structs() {
    let app = ProfileView::new(profile());
    let renamed = Rc::new(RefCell::new(None));
    let seen = Rc::clone(&renamed);
    app.on_rename(move |profile, e: events::Rename| {
        profile.set_user(e.user.clone());
        *seen.borrow_mut() = Some(e.user);
    });
    let mut driver = Driver::new(app);
    driver.start();
    driver.send(
        "Rename",
        json!({ "user": {
            "name": "Grace",
            "address": { "city": "Arlington", "zip": "22201" },
            "type": 4,
            "self": true,
        } }),
    );
    let expected = User {
        name: "Grace".to_owned(),
        address: Address {
            city: "Arlington".to_owned(),
            zip: Some("22201".to_owned()),
        },
        r#type: 4,
        self_: true,
    };
    assert_eq!(renamed.borrow().as_ref(), Some(&expected));
    assert_eq!(driver.flush().expect("user changed")["acks"], 1);
}

#[test]
fn events_of_imported_components_reach_the_root() {
    let app = ProfileView::new(profile());
    app.on_clicked(|profile, e: events::Clicked| profile.set_visits(e.count + 1));
    let mut driver = Driver::new(app);
    driver.start();
    driver.send("Clicked", json!({ "count": 3 }));
    assert_eq!(
        driver.flush(),
        Some(json!({ "patches": [[["visits"], 4]], "acks": 1 }))
    );
}

#[test]
fn components_without_a_model_pass_all_arguments() {
    let clicks = Rc::new(RefCell::new(Vec::new()));
    let seen = Rc::clone(&clicks);
    let app = Badge::new("visits".to_owned(), 7);
    app.on_clicked(move |(): &mut (), e: events::Clicked| seen.borrow_mut().push(e.count));
    let mut driver = Driver::new(app);
    assert_eq!(driver.start(), json!({ "args": ["visits", 7] }));
    driver.send("Clicked", json!({ "count": 7 }));
    assert_eq!(*clicks.borrow(), [7]);
    assert_eq!(driver.flush(), Some(json!({ "patches": [], "acks": 1 })));
}
