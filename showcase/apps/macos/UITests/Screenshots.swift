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
            app.launch()
            defer { app.terminate() }

            let button = app.buttons["page:\(id)"]
            XCTAssertTrue(button.waitForExistence(timeout: 5), "No catalog button page:\(id)")
            for _ in 0..<10 where !button.isHittable { app.scroll(byDeltaX: 0, deltaY: -250) }
            XCTAssertTrue(button.isHittable, "Catalog button page:\(id) is off screen")
            button.click()
            Thread.sleep(forTimeInterval: 1)

            let screenshot = app.windows.firstMatch.screenshot()
            let suffix = dark ? "-dark" : ""
            let attachment = XCTAttachment(data: screenshot.pngRepresentation, uniformTypeIdentifier: "public.png")
            attachment.name = "weft-\(id)\(suffix)"
            attachment.lifetime = .keepAlways
            add(attachment)
        }
    }
}
