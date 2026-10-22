"""Behavior tests against the generated Python binding and its real native library.

Run with the generated module and native library together on PYTHONPATH:
    PYTHONPATH=<generated-python-directory> python3 tests/bindings/test_pending_updates.py
"""

import typing
import unittest

import loro


class TextRoot(loro.ContainerIdLike):
    def as_container_id(self, ty):
        return loro.ContainerId.ROOT(name="text", container_type=ty)


TEXT = TextRoot()

# An accepted JSON update for B that still depends on missing A (0@1).
# Its supplied Lamport value fits u32, but the one-operation range cannot be
# represented by the binary exporter. This exercises the public import/export
# error path without a test-only native hook.
UNENCODABLE_PENDING_UPDATE = """{
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
}"""


def text(doc):
    return doc.get_text(TEXT)


def dependent_updates(base):
    """Return a snapshot before A, then separately encoded dependent A and B."""
    source = loro.LoroDoc()
    source.set_peer_id(1)
    if base:
        text(source).push_str(base)
        source.commit()
    snapshot = source.export_snapshot()
    before_a = source.oplog_vv()
    text(source).push_str("A")
    source.commit()
    update_a = source.export_updates(before_a)
    after_a = source.oplog_vv()
    text(source).push_str("B")
    source.commit()
    return source, snapshot, update_a, source.export_updates(after_a)


class PendingUpdatesBindingTests(unittest.TestCase):
    def test_optional_bytes_is_none_for_empty(self):
        self.assertEqual(
            typing.get_type_hints(loro.LoroDoc.export_pending_updates)["return"],
            typing.Optional[bytes],
        )
        self.assertIsNone(loro.LoroDoc().export_pending_updates())

    def assert_round_trip(self, base):
        source, base_snapshot, update_a, update_b = dependent_updates(base)
        receiver = loro.LoroDoc()
        receiver._import(base_snapshot)
        receiver._import(update_b)
        self.assertEqual(str(text(receiver)), base)
        applied = receiver.state_vv().encode()

        pending = receiver.export_pending_updates()
        self.assertIsInstance(pending, bytes)
        self.assertGreater(len(pending), 0)
        # Export is a read: pending remains, and no operation becomes applied.
        self.assertEqual(receiver.export_pending_updates(), pending)
        self.assertEqual(receiver.state_vv().encode(), applied)
        snapshot = receiver.export_snapshot()

        snapshot_only = loro.LoroDoc()
        snapshot_only._import(snapshot)
        self.assertIsNone(snapshot_only.export_pending_updates())
        snapshot_only._import(update_a)
        self.assertEqual(str(text(snapshot_only)), base + "A")

        restored = loro.LoroDoc()
        restored._import(snapshot)
        restored._import(pending)
        # Importing a saved pending blob again must not duplicate its operations.
        restored._import(pending)
        self.assertEqual(str(text(restored)), base)
        self.assertEqual(restored.state_vv().encode(), applied)
        self.assertIsInstance(restored.export_pending_updates(), bytes)
        restored._import(update_a)
        self.assertEqual(str(text(restored)), base + "AB")
        self.assertIsNone(restored.export_pending_updates())
        self.assertTrue(restored.oplog_vv().eq(source.oplog_vv()))
        restored._import(update_a)
        restored._import(pending)
        self.assertEqual(str(text(restored)), base + "AB")
        self.assertIsNone(restored.export_pending_updates())
        self.assertTrue(restored.oplog_vv().eq(source.oplog_vv()))

    def test_restore_with_empty_base(self):
        self.assert_round_trip("")

    def test_restore_with_nonempty_base(self):
        self.assert_round_trip("base:")

    def test_uncommitted_local_transaction_is_not_pending_updates(self):
        doc = loro.LoroDoc()
        text(doc).push_str("local")
        pending_len = doc.get_pending_txn_len()
        self.assertGreater(pending_len, 0)
        self.assertIsNone(doc.export_pending_updates())
        self.assertEqual(doc.get_pending_txn_len(), pending_len)
        self.assertEqual(str(text(doc)), "local")

    def test_export_does_not_commit_local_transaction_with_remote_pending(self):
        _, snapshot, _, update_b = dependent_updates("")
        doc = loro.LoroDoc()
        doc.set_peer_id(2)
        doc._import(snapshot)
        doc._import(update_b)
        pending = doc.export_pending_updates()
        text(doc).push_str("local")
        pending_len = doc.get_pending_txn_len()
        self.assertGreater(pending_len, 0)
        applied = doc.state_vv().encode()
        oplog = doc.oplog_vv().encode()
        self.assertEqual(doc.export_pending_updates(), pending)
        self.assertEqual(doc.get_pending_txn_len(), pending_len)
        self.assertEqual(doc.state_vv().encode(), applied)
        self.assertEqual(doc.oplog_vv().encode(), oplog)
        self.assertEqual(str(text(doc)), "local")

    def assert_encoding_error_preserves_state(self, local_edits):
        doc = loro.LoroDoc()
        doc.set_peer_id(2)
        empty = doc.oplog_vv().encode()
        self.assertEqual(doc.state_vv().encode(), empty)
        self.assertEqual(doc.get_pending_txn_len(), 0)
        status = doc.import_json_updates(UNENCODABLE_PENDING_UPDATE)
        self.assertEqual(status.success, {})
        self.assertEqual(status.pending, {1: loro.CounterSpan(start=1, end=2)})
        self.assertEqual(doc.state_vv().encode(), empty)
        self.assertEqual(doc.oplog_vv().encode(), empty)
        self.assertEqual(doc.get_pending_txn_len(), 0)
        if local_edits:
            text(doc).push_str("local")
        pending_len = doc.get_pending_txn_len()
        if local_edits:
            self.assertGreater(pending_len, 0)
        else:
            self.assertEqual(pending_len, 0)
        applied = doc.state_vv().encode()
        oplog = doc.oplog_vv().encode()

        # A missing UDL variant becomes a UniFFI runtime panic error instead;
        # assert the generated domain-error class and its native Display text.
        # Repeating the call also proves that a failed export retains pending.
        for _ in range(2):
            with self.assertRaises(loro.LoroEncodeError.InternalError) as caught:
                doc.export_pending_updates()
            self.assertEqual(
                str(caught.exception),
                "Export failed: pending change's supplied Lamport range cannot be encoded",
            )
            self.assertEqual(doc.get_pending_txn_len(), pending_len)
            self.assertEqual(doc.state_vv().encode(), applied)
            self.assertEqual(doc.oplog_vv().encode(), oplog)
            if local_edits:
                self.assertEqual(str(text(doc)), "local")

    def test_encoding_error_is_typed_and_preserves_empty_state(self):
        self.assert_encoding_error_preserves_state(local_edits=False)

    def test_encoding_error_is_typed_and_preserves_local_transaction(self):
        self.assert_encoding_error_preserves_state(local_edits=True)


if __name__ == "__main__":
    unittest.main(verbosity=2)
