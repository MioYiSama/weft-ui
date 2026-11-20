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
        }
    }
}
