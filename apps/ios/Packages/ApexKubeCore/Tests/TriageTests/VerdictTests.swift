import XCTest
@testable import Triage

final class VerdictTests: XCTestCase {
    func testStatusWireNamesMatchSpec() {
        XCTAssertEqual(VerdictStatus.allCases.map(\.rawValue), ["ok", "warn", "fail", "unknown"])
    }
}
