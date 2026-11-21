import XCTest

final class Screenshots: XCTestCase {
    @MainActor
    func testPages() throws {
        continueAfterFailure = false
        let env = ProcessInfo.processInfo.environment
        guard let cases = env["WEFT_PAGES"] else { throw XCTSkip("Run pnpm --filter showcase shot:macos") }
        let pages = cases.split(separator: ",")

        for page in pages {
            let parts = page.split(separator: ":")
            let id = parts[0]
            let dark = parts[1] == "dark"
            let app = XCUIApplication()
            app.launchEnvironment["WEFT_SHOT_APPEARANCE"] = dark ? "dark" : "light"
            app.launchArguments = ["-NSQuitAlwaysKeepsWindows", "NO"]
            app.launch()
            defer { app.terminate() }
            app.activate()

            let window = app.windows.firstMatch
            if !window.exists { app.typeKey("n", modifierFlags: .command) }
            XCTAssertTrue(window.waitForExistence(timeout: 5), "No showcase window")
            let button = window.buttons["page:\(id)"]
            XCTAssertTrue(button.waitForExistence(timeout: 5), "No catalog button page:\(id)")
            let catalog = window.scrollViews.firstMatch
            for _ in 0..<10 where !button.isHittable { catalog.scroll(byDeltaX: 0, deltaY: -250) }
            XCTAssertTrue(button.isHittable, "Catalog button page:\(id) is off screen")
            button.click()
            Thread.sleep(forTimeInterval: 1)
            if id == "controls" { XCTAssertTrue(window.staticTexts["控件设置"].exists) }

            let screenshot = window.screenshot()
            let suffix = dark ? "-dark" : ""
            let attachment = XCTAttachment(data: screenshot.pngRepresentation, uniformTypeIdentifier: "public.png")
            attachment.name = "weft-\(id)\(suffix)"
            attachment.lifetime = .keepAlways
            add(attachment)
        }
    }
}
