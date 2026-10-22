use std::{collections::HashMap, sync::Arc};

use loro_ffi::{LoroDoc, LoroEncodeError, LoroText, VersionVector};

fn text(doc: &LoroDoc, name: &str) -> Arc<LoroText> {
    doc.get_text(Arc::new(name.to_owned()))
}

// Only the public FFI wrapper is used here, including for constructing the
// out-of-order updates. B depends on A and must never apply without it.
struct OutOfOrderUpdates {
    base_snapshot: Vec<u8>,
    a: Vec<u8>,
    b: Vec<u8>,
}

fn out_of_order_updates(base: &str) -> OutOfOrderUpdates {
    let source = LoroDoc::new();
    source.set_peer_id(1).unwrap();
    let source_text = text(&source, "text");
    source_text.insert(0, base).unwrap();
    source.commit();
    let base_snapshot = source.export_snapshot().unwrap();
    let before_a = source.oplog_vv();

    source_text.insert(base.len() as u32, "A").unwrap();
    source.commit();
    let a = source.export_updates(&before_a).unwrap();
    let before_b = source.oplog_vv();
    source_text.insert(base.len() as u32 + 1, "B").unwrap();
    source.commit();
    let b = source.export_updates(&before_b).unwrap();

    OutOfOrderUpdates {
        base_snapshot,
        a,
        b,
    }
}

#[derive(Debug, PartialEq)]
struct ObservedState {
    oplog_vv: HashMap<u64, i32>,
    state_vv: HashMap<u64, i32>,
    oplog_frontiers: Vec<u8>,
    state_frontiers: Vec<u8>,
    pending_txn_len: u32,
    len_ops: u64,
    len_changes: u64,
}

fn observe(doc: &LoroDoc) -> ObservedState {
    ObservedState {
        oplog_vv: doc.oplog_vv().to_hashmap(),
        state_vv: doc.state_vv().to_hashmap(),
        oplog_frontiers: doc.oplog_frontiers().encode(),
        state_frontiers: doc.state_frontiers().encode(),
        pending_txn_len: doc.get_pending_txn_len(),
        len_ops: doc.len_ops(),
        len_changes: doc.len_changes(),
    }
}

fn restore_checkpoint(doc: &LoroDoc) -> LoroDoc {
    // Keep both fields from the same document state; nothing mutates or imports
    // between these two exports.
    let snapshot = doc.export_snapshot().unwrap();
    let pending = doc.export_pending_updates().unwrap();
    let restored = LoroDoc::new();
    restored.import(&snapshot).unwrap();
    if let Some(pending) = pending {
        restored.import(&pending).unwrap();
    }
    restored
}

#[test]
fn empty_and_fully_applied_documents_have_no_pending_updates() {
    let doc = LoroDoc::new();
    assert_eq!(doc.export_pending_updates().unwrap(), None);

    text(&doc, "text").insert(0, "committed").unwrap();
    doc.commit();
    assert_eq!(doc.export_pending_updates().unwrap(), None);

    let restored = LoroDoc::new();
    restored
        .import(&doc.export_updates(&VersionVector::new()).unwrap())
        .unwrap();
    assert_eq!(text(&restored, "text").to_string(), "committed");
    assert_eq!(restored.export_pending_updates().unwrap(), None);
}

fn check_out_of_order_roundtrip(base: &str) {
    let updates = out_of_order_updates(base);
    let receiver = LoroDoc::new();
    receiver.import(&updates.base_snapshot).unwrap();
    receiver.import(&updates.b).unwrap();
    assert_eq!(text(&receiver, "text").to_string(), base);

    let before = observe(&receiver);
    let pending = receiver.export_pending_updates().unwrap().unwrap();
    assert!(!pending.is_empty());
    assert_eq!(observe(&receiver), before);
    assert!(receiver.export_pending_updates().unwrap().is_some());
    assert_eq!(observe(&receiver), before);
    assert_eq!(text(&receiver, "text").to_string(), base);

    // Ordinary snapshots and updates omit B. Importing A into either therefore
    // produces only A, proving that the separately saved bytes are necessary.
    for ordinary in [
        receiver.export_snapshot().unwrap(),
        receiver.export_updates(&VersionVector::new()).unwrap(),
    ] {
        let without_pending = LoroDoc::new();
        without_pending.import(&ordinary).unwrap();
        assert_eq!(without_pending.export_pending_updates().unwrap(), None);
        without_pending.import(&updates.a).unwrap();
        assert_eq!(
            text(&without_pending, "text").to_string(),
            format!("{base}A")
        );
    }

    let restored = restore_checkpoint(&receiver);
    assert_eq!(text(&restored, "text").to_string(), base);
    assert!(restored.export_pending_updates().unwrap().is_some());
    assert_eq!(observe(&restored), before);

    // A second save/restore while A is still missing must preserve B as well.
    let restored_again = restore_checkpoint(&restored);
    assert_eq!(text(&restored_again, "text").to_string(), base);
    restored_again.import(&pending).unwrap();
    restored_again.import(&pending).unwrap();
    assert_eq!(observe(&restored_again), before);
    assert!(restored_again.export_pending_updates().unwrap().is_some());

    for target in [&receiver, &restored, &restored_again] {
        target.import(&updates.a).unwrap();
        assert_eq!(text(target, "text").to_string(), format!("{base}AB"));
        // Persisting this None must clear an older saved pending record.
        assert_eq!(target.export_pending_updates().unwrap(), None);
        let resolved = observe(target);

        target.import(&pending).unwrap();
        target.import(&updates.a).unwrap();
        target.import(&updates.b).unwrap();
        assert_eq!(text(target, "text").to_string(), format!("{base}AB"));
        assert_eq!(observe(target), resolved);
        assert_eq!(target.export_pending_updates().unwrap(), None);
    }
}

#[test]
fn pending_updates_roundtrip_from_an_empty_base() {
    check_out_of_order_roundtrip("");
}

#[test]
fn pending_updates_roundtrip_with_a_nonempty_base() {
    check_out_of_order_roundtrip("base|");
}

#[test]
fn exporting_no_remote_pending_does_not_commit_local_edits() {
    let doc = LoroDoc::new();
    text(&doc, "draft").insert(0, "uncommitted").unwrap();
    let before = observe(&doc);
    assert!(before.pending_txn_len > 0);

    assert_eq!(doc.export_pending_updates().unwrap(), None);
    assert_eq!(observe(&doc), before);
    assert_eq!(text(&doc, "draft").to_string(), "uncommitted");

    doc.commit();
    assert_eq!(doc.get_pending_txn_len(), 0);
    assert_eq!(text(&doc, "draft").to_string(), "uncommitted");
}

#[test]
fn exporting_remote_pending_does_not_commit_or_include_local_edits() {
    let updates = out_of_order_updates("");
    let receiver = LoroDoc::new();
    receiver.set_peer_id(2).unwrap();
    receiver.import(&updates.b).unwrap();
    text(&receiver, "draft").insert(0, "uncommitted").unwrap();
    let before = observe(&receiver);
    assert!(before.pending_txn_len > 0);

    let pending = receiver.export_pending_updates().unwrap().unwrap();
    assert_eq!(observe(&receiver), before);
    assert_eq!(text(&receiver, "draft").to_string(), "uncommitted");
    assert!(receiver.export_pending_updates().unwrap().is_some());
    assert_eq!(observe(&receiver), before);

    let restored = LoroDoc::new();
    restored.import(&pending).unwrap();
    assert_eq!(text(&restored, "text").to_string(), "");
    restored.import(&updates.a).unwrap();
    assert_eq!(text(&restored, "text").to_string(), "AB");
    assert!(text(&restored, "draft").is_empty());

    receiver.commit();
    assert_eq!(receiver.get_pending_txn_len(), 0);
    assert_eq!(text(&receiver, "draft").to_string(), "uncommitted");
    receiver.import(&updates.a).unwrap();
    assert_eq!(text(&receiver, "text").to_string(), "AB");
    assert_eq!(receiver.export_pending_updates().unwrap(), None);
}

#[test]
fn unrepresentable_pending_lamport_returns_internal_error_without_mutation() {
    // The public JSON importer accepts this supplied Lamport while dependency
    // 0@1 is missing, but the pending operation's range cannot fit in binary.
    let json = r#"{
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
    }"#;
    let receiver = LoroDoc::new();
    let status = receiver.import_json_updates(json).unwrap();
    assert!(status.success.is_empty());
    let pending = status.pending.unwrap();
    assert_eq!(pending.len(), 1);
    let span = pending.get(&1).unwrap();
    assert_eq!((span.start, span.end), (1, 2));
    let before = observe(&receiver);
    assert!(before.oplog_vv.is_empty());
    assert!(before.state_vv.is_empty());
    assert_eq!(before.pending_txn_len, 0);

    for _ in 0..2 {
        match receiver.export_pending_updates() {
            Err(LoroEncodeError::InternalError(message)) => assert_eq!(
                message.as_ref(),
                "pending change's supplied Lamport range cannot be encoded"
            ),
            result => panic!("expected InternalError, got {result:?}"),
        }
        assert_eq!(observe(&receiver), before);
        assert!(receiver.oplog_vv().to_hashmap().is_empty());
        assert!(receiver.state_vv().to_hashmap().is_empty());
        assert_eq!(receiver.get_pending_txn_len(), 0);
    }
}
