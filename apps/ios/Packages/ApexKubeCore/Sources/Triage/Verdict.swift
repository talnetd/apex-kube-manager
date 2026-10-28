import Foundation

/// Health of a single object as decided by a triage rule.
/// Raw values are the wire names shared with Android and desktop
/// (see docs/mobile/architecture.md, section 3).
public enum VerdictStatus: String, Codable, CaseIterable, Sendable {
    case ok
    case warn
    case fail
    case unknown
}

/// Output of a triage rule. Field names match the shared spec.
public struct Verdict: Codable, Equatable, Sendable {
    public let status: VerdictStatus
    public let reason: String
    public let ruleId: String
    public let since: Date?

    public init(status: VerdictStatus, reason: String, ruleId: String, since: Date? = nil) {
        self.status = status
        self.reason = reason
        self.ruleId = ruleId
        self.since = since
    }
}
