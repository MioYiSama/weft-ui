import XCTest

final class Screenshots: XCTestCase {
    @MainActor
    func testPages() throws {
        continueAfterFailure = false
        let env = ProcessInfo.processInfo.environment
        guard let cases = env["WEFT_PAGES"] else { throw XCTSkip("Run pnpm --filter showcase shot:ios") }
        let pages = cases.split(separator: ",")
        let directory = URL(fileURLWithPath: try XCTUnwrap(env["WEFT_SHOT_DIR"]), isDirectory: true)
        let suffix = env["WEFT_APPEARANCE"] == "dark" ? "-dark" : ""

        for id in pages {
            let app = XCUIApplication()
            app.launch()
            defer { app.terminate() }

            let button = app.buttons["page:\(id)"]
            XCTAssertTrue(button.waitForExistence(timeout: 5), "No catalog button page:\(id)")
            for _ in 0..<10 where !button.isHittable { app.swipeUp() }
            XCTAssertTrue(button.isHittable, "Catalog button page:\(id) is off screen")
            button.tap()
            Thread.sleep(forTimeInterval: 1)

            let screenshot = XCUIScreen.main.screenshot()
            try screenshot.pngRepresentation.write(to: directory.appendingPathComponent("\(id)\(suffix).png"))
            if id == "appearance" { checkVisibility(in: app) }
            if id == "controls" { checkControls(in: app) }
            if id == "inputs" { checkInputs(in: app) }
        }
    }

    @MainActor
    private func checkInputs(in app: XCUIApplication) {
        let field = app.textFields["搜索，提交后清空"]
        let password = app.secureTextFields["输入密码"]
        let request = app.buttons["聚焦文本框"]
        let release = app.buttons["放弃焦点"]
        let focused = app.staticTexts["已聚焦"]
        let blurred = app.staticTexts["未聚焦"]

        XCTAssertTrue(blurred.exists)
        request.tap()
        XCTAssertTrue(focused.waitForExistence(timeout: 5))
        release.tap()
        XCTAssertTrue(blurred.waitForExistence(timeout: 5))

        field.tap()
        XCTAssertTrue(focused.waitForExistence(timeout: 5))
        XCTAssertTrue(request.exists, "User focus must not write back to focused")
        request.tap()
        password.tap()
        XCTAssertTrue(blurred.waitForExistence(timeout: 5))
        XCTAssertTrue(release.exists, "User blur must not write back or refocus the field")
        release.tap()

        request.tap()
        XCTAssertTrue(focused.waitForExistence(timeout: 5))
        let prefix = String(repeating: "a", count: 19)
        field.typeText(prefix + "🙂")
        XCTAssertEqual(field.value as? String, prefix)
        field.typeText("bcd")
        XCTAssertEqual(field.value as? String, prefix + "b")
        let search = app.keyboards.buttons["Search"]
        XCTAssertTrue(search.exists)
        search.tap()
        XCTAssertTrue(app.staticTexts["0/20"].waitForExistence(timeout: 5))
        XCTAssertEqual(field.value as? String, "搜索，提交后清空")
    }

    @MainActor
    private func checkControls(in app: XCUIApplication) {
        let lock = app.switches.element(boundBy: 0)
        let alerts = app.switches.element(boundBy: 1)
        let sounds = app.switches.element(boundBy: 2)
        let checkbox = app.buttons.matching(identifier: "Checkbox").firstMatch
        let slider = app.sliders.firstMatch
        let notes = app.textFields.firstMatch
        let region = app.buttons["本地"]
        let channel = app.buttons["选择频道"]

        XCTAssertTrue(app.activityIndicators.firstMatch.exists)
        XCTAssertTrue(app.progressIndicators.matching(NSPredicate(format: "value == %@", "30%")).firstMatch.exists)
        XCTAssertTrue(app.progressIndicators.matching(NSPredicate(format: "value == %@", "80%")).firstMatch.exists)
        XCTAssertFalse(app.switches.element(boundBy: 3).isEnabled)
        XCTAssertFalse(app.buttons.matching(identifier: "Checkbox").element(boundBy: 1).isEnabled)
        XCTAssertFalse(app.sliders.element(boundBy: 1).isEnabled)
        XCTAssertFalse(app.buttons["不可更改"].isEnabled)
        XCTAssertFalse(app.textFields.element(boundBy: 1).isEnabled)

        XCTAssertEqual(lock.value as? String, "0")
        XCTAssertEqual(alerts.value as? String, "1")
        alerts.tap()
        XCTAssertEqual(alerts.value as? String, "0")
        sounds.tap()
        XCTAssertEqual(sounds.value as? String, "1")
        checkbox.tap()
        XCTAssertEqual(checkbox.value as? String, "0")
        slider.adjust(toNormalizedSliderPosition: 0.8)
        XCTAssertFalse(app.staticTexts["音量 · 40%"].exists)
        let volume = slider.value as? String ?? ""
        XCTAssertTrue(app.staticTexts["音量 · \(volume)"].exists)
        region.tap()
        app.buttons["远程"].tap()
        XCTAssertTrue(app.buttons["远程"].exists)
        XCTAssertTrue(channel.exists)
        notes.tap()
        notes.typeText("x\nz")
        XCTAssertEqual(notes.value as? String, "x\nz")

        lock.tap()
        XCTAssertEqual(lock.value as? String, "1")
        alerts.tap()
        XCTAssertEqual(alerts.value as? String, "0")
        sounds.tap()
        XCTAssertEqual(sounds.value as? String, "1")
        checkbox.tap()
        XCTAssertEqual(checkbox.value as? String, "0")
        slider.adjust(toNormalizedSliderPosition: 0.2)
        XCTAssertEqual(slider.value as? String, volume)
        XCTAssertTrue(app.staticTexts["音量 · \(volume)"].exists)
        app.buttons["远程"].tap()
        app.buttons["本地"].tap()
        XCTAssertTrue(app.buttons["远程"].exists)
        channel.tap()
        app.buttons["稳定版"].tap()
        XCTAssertTrue(channel.exists)
        notes.tap()
        notes.typeText("y")
        XCTAssertEqual(notes.value as? String, "x\nz")
    }

    @MainActor
    private func checkVisibility(in app: XCUIApplication) {
        let card = app.staticTexts["This card leaves no gap when hidden."]
        let end = app.staticTexts["End of grid"]
        XCTAssertTrue(card.exists)
        XCTAssertTrue(end.exists)
        let shownEnd = end.frame.minY
        let cardHeight = card.frame.height

        app.buttons["Hide card"].tap()
        XCTAssertTrue(card.waitForNonExistence(timeout: 5))
        XCTAssertGreaterThan(shownEnd - end.frame.minY, cardHeight)

        app.buttons["Show card"].tap()
        XCTAssertTrue(card.waitForExistence(timeout: 5))
        XCTAssertEqual(end.frame.minY, shownEnd, accuracy: 1)
    }
}
