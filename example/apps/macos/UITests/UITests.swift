import XCTest

/// The example's click flow over every built-in element; run with
/// `pnpm --filter example test:macos`.
final class UITests: XCTestCase {
    @MainActor
    func testClickFlow() throws {
        let app = XCUIApplication()
        app.launch()
        let window = app.windows.firstMatch
        XCTAssertTrue(window.waitForExistence(timeout: 10))

        // Button and `if`.
        window.buttons["Click me"].click()
        XCTAssertTrue(window.staticTexts["You clicked!"].waitForExistence(timeout: 5))

        // A controlled input: drafts longer than 12 characters are refused and rolled back.
        let field = window.textFields.firstMatch
        let add = window.buttons["Add"]
        XCTAssertFalse(add.isEnabled)
        field.click()
        field.typeText("Buy milk and bread")
        XCTAssertEqual(field.value as? String, "Buy milk and")
        XCTAssertTrue(add.isEnabled)

        // `for` rows.
        add.click()
        XCTAssertTrue(window.staticTexts["Buy milk and"].waitForExistence(timeout: 5))
        XCTAssertEqual(field.value as? String, "")
        field.typeText("Eggs")
        add.click()
        XCTAssertTrue(window.staticTexts["2 todos"].waitForExistence(timeout: 5))

        // Checkboxes in the rows.
        let first = window.checkBoxes.element(boundBy: 0)
        XCTAssertEqual(first.value as? Int, 0)
        first.click()
        XCTAssertEqual(first.value as? Int, 1)
        XCTAssertEqual(window.checkBoxes.element(boundBy: 1).value as? Int, 0)

        // Removing the first row keeps the second.
        window.buttons.matching(identifier: "Remove").element(boundBy: 0).click()
        XCTAssertTrue(window.staticTexts["1 todos"].waitForExistence(timeout: 5))
        XCTAssertFalse(window.staticTexts["Buy milk and"].exists)
        XCTAssertTrue(window.staticTexts["Eggs"].exists)
        XCTAssertEqual(window.checkBoxes.element(boundBy: 0).value as? Int, 0)
    }
}
