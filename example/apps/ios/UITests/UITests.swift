import XCTest

/// The example's click flow over every built-in element; run with
/// `pnpm --filter example test:ios`.
final class UITests: XCTestCase {
    @MainActor
    func testClickFlow() throws {
        let app = XCUIApplication()
        app.launch()

        // Button and `if`.
        app.buttons["Click me"].tap()
        XCTAssertTrue(app.staticTexts["You clicked!"].waitForExistence(timeout: 5))

        // A controlled input: drafts longer than 12 characters are refused and rolled back.
        let field = app.textFields["New todo"]
        let add = app.buttons["Add"]
        XCTAssertFalse(add.isEnabled)
        field.tap()
        field.typeText("Buy milk and bread")
        XCTAssertEqual(field.value as? String, "Buy milk and")
        XCTAssertTrue(add.isEnabled)

        // `for` rows.
        add.tap()
        XCTAssertTrue(app.staticTexts["Buy milk and"].waitForExistence(timeout: 5))
        field.typeText("Eggs")
        add.tap()
        XCTAssertTrue(app.staticTexts["2 todos"].waitForExistence(timeout: 5))

        // Checkboxes (switches on iOS) in the rows.
        let first = app.switches.element(boundBy: 0)
        XCTAssertEqual(first.value as? String, "0")
        first.tap()
        XCTAssertEqual(first.value as? String, "1")
        XCTAssertEqual(app.switches.element(boundBy: 1).value as? String, "0")

        // Removing the first row keeps the second.
        app.buttons.matching(identifier: "Remove").element(boundBy: 0).tap()
        XCTAssertTrue(app.staticTexts["1 todos"].waitForExistence(timeout: 5))
        XCTAssertFalse(app.staticTexts["Buy milk and"].exists)
        XCTAssertTrue(app.staticTexts["Eggs"].exists)
        XCTAssertEqual(app.switches.element(boundBy: 0).value as? String, "0")
    }
}
