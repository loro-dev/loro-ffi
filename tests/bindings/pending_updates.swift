// Compile together with the generated loro.swift and link its native library.
import Foundation

private final class TextRoot: ContainerIdLike, @unchecked Sendable {
    func asContainerId(ty: ContainerType) -> ContainerId {
        .root(name: "text", containerType: ty)
    }
}

private let textRoot = TextRoot()

private func text(_ doc: LoroDoc) -> LoroText {
    doc.getText(id: textRoot)
}

private func expect(_ condition: Bool, _ message: String) {
    precondition(condition, message)
}

private func dependentUpdates(_ base: String) throws -> (LoroDoc, Data, Data, Data) {
    let source = LoroDoc()
    try source.setPeerId(peer: 1)
    if !base.isEmpty {
        try text(source).pushStr(s: base)
        source.commit()
    }
    let snapshot = try source.exportSnapshot()
    let beforeA = source.oplogVv()
    try text(source).pushStr(s: "A")
    source.commit()
    let updateA = try source.exportUpdates(vv: beforeA)
    let afterA = source.oplogVv()
    try text(source).pushStr(s: "B")
    source.commit()
    return (source, snapshot, updateA, try source.exportUpdates(vv: afterA))
}

private func roundTrip(_ base: String) throws {
    let (source, baseSnapshot, updateA, updateB) = try dependentUpdates(base)
    let receiver = LoroDoc()
    _ = try receiver.`import`(bytes: baseSnapshot)
    _ = try receiver.`import`(bytes: updateB)
    expect(text(receiver).description == base, "B must wait for A")
    let applied = receiver.stateVv().encode()

    // This annotation also verifies the generated native optional-bytes type.
    let optionalPending: Data? = try receiver.exportPendingUpdates()
    guard let pending = optionalPending else {
        preconditionFailure("missing dependencies must produce pending bytes")
    }
    expect(!pending.isEmpty, "Some bytes must be a nonempty encoded blob")
    let repeated = try receiver.exportPendingUpdates()
    expect(repeated == pending, "export must preserve pending operations")
    expect(receiver.stateVv().encode() == applied, "export must not advance applied VV")
    let snapshot = try receiver.exportSnapshot()

    let snapshotOnly = LoroDoc()
    _ = try snapshotOnly.`import`(bytes: snapshot)
    let absent = try snapshotOnly.exportPendingUpdates()
    expect(absent == nil, "ordinary snapshot must not retain pending operations")
    _ = try snapshotOnly.`import`(bytes: updateA)
    expect(text(snapshotOnly).description == base + "A", "snapshot alone must omit B")

    let restored = LoroDoc()
    _ = try restored.`import`(bytes: snapshot)
    _ = try restored.`import`(bytes: pending)
    _ = try restored.`import`(bytes: pending)
    expect(text(restored).description == base, "restored B must still wait for real A")
    expect(restored.stateVv().encode() == applied, "pending import must not advance applied VV")
    _ = try restored.`import`(bytes: updateA)
    expect(text(restored).description == base + "AB", "restored B must apply once after A")
    let cleared = try restored.exportPendingUpdates()
    expect(cleared == nil, "resolved pending must be native nil")
    expect(restored.oplogVv().eq(other: source.oplogVv()), "restored history must match source")
    _ = try restored.`import`(bytes: updateA)
    _ = try restored.`import`(bytes: pending)
    expect(text(restored).description == base + "AB", "repeated imports must be idempotent")
    let stillCleared = try restored.exportPendingUpdates()
    expect(stillCleared == nil, "repeated applied imports must not recreate pending")
}

private func uncommittedTransaction() throws {
    let doc = LoroDoc()
    let empty: Data? = try doc.exportPendingUpdates()
    expect(empty == nil, "empty pending must be nil, not empty Data")
    try text(doc).pushStr(s: "local")
    let transactionLength = doc.getPendingTxnLen()
    expect(transactionLength > 0, "local change must remain uncommitted")
    let pending = try doc.exportPendingUpdates()
    expect(pending == nil, "uncommitted local changes are not remote pending operations")
    expect(doc.getPendingTxnLen() == transactionLength, "export must not commit local transaction")

    let (_, snapshot, _, updateB) = try dependentUpdates("")
    let receiver = LoroDoc()
    try receiver.setPeerId(peer: 2)
    _ = try receiver.`import`(bytes: snapshot)
    _ = try receiver.`import`(bytes: updateB)
    let remotePending = try receiver.exportPendingUpdates()
    try text(receiver).pushStr(s: "local")
    let localLength = receiver.getPendingTxnLen()
    let applied = receiver.stateVv().encode()
    let oplog = receiver.oplogVv().encode()
    let exported = try receiver.exportPendingUpdates()
    expect(exported == remotePending, "local edits must not enter remote pending blob")
    expect(receiver.getPendingTxnLen() == localLength && localLength > 0, "export must preserve local transaction")
    expect(receiver.stateVv().encode() == applied, "export must preserve applied VV")
    expect(receiver.oplogVv().encode() == oplog, "export must preserve committed history")
    expect(text(receiver).description == "local", "export must preserve local text")
}

private func encodingError(_ localEdits: Bool) throws {
    // Accepted JSON input for B still depends on missing A (0@1). Its supplied
    // Lamport fits u32 but its one-operation range cannot be encoded in binary.
    // Use the real public importer/exporter, without a native test-only hook.
    let json = """
    {
        "schema_version": 1,
        "start_version": {"1": 0},
        "peers": null,
        "changes": [{
            "id": "1@1", "timestamp": 0, "deps": ["0@1"],
            "lamport": 4294967295, "msg": null,
            "ops": [{
                "container": "cid:root-map:Map",
                "content": {"type": "insert", "key": "b", "value": true},
                "counter": 1
            }]
        }]
    }
    """
    let doc = LoroDoc()
    try doc.setPeerId(peer: 2)
    let empty = doc.oplogVv().encode()
    expect(doc.stateVv().encode() == empty, "fresh state VV must be empty")
    expect(doc.getPendingTxnLen() == 0, "fresh local transaction must be empty")
    let status = try doc.importJsonUpdates(json: json)
    expect(status.success.isEmpty, "no imported operation may be accepted as applied")
    expect(status.pending == [1: CounterSpan(start: 1, end: 2)], "public import must retain precisely B as pending")
    expect(doc.stateVv().encode() == empty, "pending import must leave state VV empty")
    expect(doc.oplogVv().encode() == empty, "pending import must leave oplog VV empty")
    expect(doc.getPendingTxnLen() == 0, "pending import must leave local transaction empty")
    if localEdits {
        try text(doc).pushStr(s: "local")
    }
    let transactionLength = doc.getPendingTxnLen()
    expect(localEdits ? transactionLength > 0 : transactionLength == 0, "local transaction length must match edits")
    let applied = doc.stateVv().encode()
    let oplog = doc.oplogVv().encode()

    // Only the generated typed domain-error case is accepted. A UniFFI panic
    // error escapes this catch and fails the executable. Repeat the call to
    // verify that a failed export leaves the pending input in place.
    for _ in 0..<2 {
        do {
            _ = try doc.exportPendingUpdates()
            preconditionFailure("unrepresentable pending range must throw")
        } catch LoroEncodeError.InternalError(let message) {
            expect(
                message == "Export failed: pending change's supplied Lamport range cannot be encoded",
                "the core error's Display text must survive error lowering"
            )
        }
        expect(doc.getPendingTxnLen() == transactionLength, "failed export must preserve local transaction")
        expect(doc.stateVv().encode() == applied, "failed export must preserve applied VV")
        expect(doc.oplogVv().encode() == oplog, "failed export must preserve committed history")
        if localEdits {
            expect(text(doc).description == "local", "failed export must preserve local text")
        }
    }
}

@main
private enum PendingUpdatesBindingTests {
    static func main() throws {
        try roundTrip("")
        try roundTrip("base:")
        try uncommittedTransaction()
        try encodingError(false)
        try encodingError(true)
        print("Swift pending updates binding tests passed")
    }
}
