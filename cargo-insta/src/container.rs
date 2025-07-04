use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) use insta::TextSnapshotKind;
use insta::_cargo_insta_support::{ContentError, PendingInlineSnapshot};
use insta::{internals::SnapshotContents, Snapshot};

use crate::inline::FilePatcher;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Operation {
    Accept,
    AcceptAll,
    Reject,
    RejectAll,
    Skip,
    SkipAll,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingSnapshot {
    #[allow(dead_code)]
    id: usize,
    pub(crate) old: Option<Snapshot>,
    pub(crate) new: Snapshot,
    pub(crate) op: Operation,
    pub(crate) line: Option<u32>,
}

impl PendingSnapshot {
    pub(crate) fn summary(&self) -> String {
        use std::fmt::Write;
        let mut rv = String::new();
        if let Some(source) = self.new.metadata().source() {
            write!(&mut rv, "{source}").unwrap();
        }
        if let Some(line) = self.line {
            write!(&mut rv, ":{line}").unwrap();
        }
        if let Some(name) = self.new.snapshot_name() {
            write!(&mut rv, " ({name})").unwrap();
        }
        rv
    }
}

/// A snapshot and its immediate context, which loads & saves the snapshot. It
/// holds either a single file snapshot, or all the inline snapshots from a
/// single rust file.
#[derive(Debug, Clone)]
pub(crate) struct SnapshotContainer {
    // Path of the pending snapshot file (generally a `.snap.new` or `.pending-snap` file)
    pending_path: PathBuf,
    // Path of the target snapshot file (generally a `.snap` file)
    target_path: PathBuf,
    kind: TextSnapshotKind,
    snapshots: Vec<PendingSnapshot>,
    patcher: Option<FilePatcher>,
}

impl SnapshotContainer {
    pub(crate) fn load(
        pending_path: PathBuf,
        target_path: PathBuf,
        kind: TextSnapshotKind,
    ) -> Result<SnapshotContainer, Box<dyn Error>> {
        let mut snapshots = Vec::new();
        let patcher = match kind {
            TextSnapshotKind::File => {
                let old = if fs::metadata(&target_path).is_err() {
                    None
                } else {
                    Some(Snapshot::from_file(&target_path)?)
                };
                let new = Snapshot::from_file(&pending_path)?;
                snapshots.push(PendingSnapshot {
                    id: 0,
                    old,
                    new,
                    op: Operation::Skip,
                    line: None,
                });
                None
            }
            TextSnapshotKind::Inline => {
                let mut pending_vec = PendingInlineSnapshot::load_batch(&pending_path)?;
                let mut have_new = false;

                let rv = if fs::metadata(&target_path).is_ok() {
                    let mut patcher = FilePatcher::open(&target_path)?;
                    pending_vec.sort_by_key(|pending| pending.line);
                    for (id, pending) in pending_vec.into_iter().enumerate() {
                        if let Some(new) = pending.new {
                            if patcher.add_snapshot_macro(pending.line as usize) {
                                snapshots.push(PendingSnapshot {
                                    id,
                                    old: pending.old,
                                    new,
                                    op: Operation::Skip,
                                    line: Some(pending.line),
                                });
                                have_new = true;
                            } else {
                                // this is an outdated snapshot and the file changed.
                            }
                        }
                    }
                    Some(patcher)
                } else {
                    None
                };

                // if we don't actually have any new pending we better delete the file.
                // this can happen if the test code left a stale snapshot behind.
                // The runtime code will issue something like this:
                //   PendingInlineSnapshot::new(None, None, line).save(pending_snapshots)?;
                if !have_new {
                    fs::remove_file(&pending_path)
                        .map_err(|e| ContentError::FileIo(e, pending_path.to_path_buf()))?;
                }

                rv
            }
        };

        Ok(SnapshotContainer {
            pending_path,
            target_path,
            kind,
            snapshots,
            patcher,
        })
    }

    pub(crate) fn target_file(&self) -> &Path {
        &self.target_path
    }

    pub(crate) fn snapshot_file(&self) -> Option<&Path> {
        match self.kind {
            TextSnapshotKind::File => Some(&self.target_path),
            TextSnapshotKind::Inline => None,
        }
    }

    pub(crate) fn snapshot_sort_key(&self) -> impl Ord + '_ {
        let path = self
            .pending_path
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or_default();
        let mut pieces = path.rsplitn(2, '-');
        if let Some(num_suffix) = pieces.next().and_then(|x| x.parse::<i64>().ok()) {
            (pieces.next().unwrap_or(""), num_suffix)
        } else {
            (path, 0)
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.snapshots.len()
    }

    pub(crate) fn iter_snapshots(&mut self) -> impl Iterator<Item = &'_ mut PendingSnapshot> {
        self.snapshots.iter_mut()
    }

    pub(crate) fn commit(&mut self) -> Result<(), Box<dyn Error>> {
        // Try removing the snapshot file. If it fails, it's
        // likely because it another process removed it; which
        // is fine — print a message and continue.
        let try_removing_snapshot = |p: &Path| {
            fs::remove_file(p).unwrap_or_else(|_| {
                    eprintln!(
                        "Pending snapshot file at {p:?} couldn't be removed. It was likely removed by another process."
                    );
                });
        };

        if let Some(ref mut patcher) = self.patcher {
            let mut new_pending = vec![];
            let mut did_accept = false;
            let mut did_skip = false;

            for (idx, snapshot) in self.snapshots.iter().enumerate() {
                match snapshot.op {
                    Operation::Accept | Operation::AcceptAll => {
                        patcher.set_new_content(
                            idx,
                            match snapshot.new.contents() {
                                SnapshotContents::Text(c) => c,
                                _ => unreachable!(),
                            },
                        );
                        did_accept = true;
                    }
                    Operation::Reject | Operation::RejectAll => {}
                    Operation::Skip | Operation::SkipAll => {
                        new_pending.push(PendingInlineSnapshot::new(
                            Some(snapshot.new.clone()),
                            snapshot.old.clone(),
                            patcher.get_new_line(idx) as u32,
                        ));
                        did_skip = true;
                    }
                }
            }

            if did_accept {
                patcher.save()?;
            }
            if did_skip {
                PendingInlineSnapshot::save_batch(&self.pending_path, &new_pending)?;
            } else {
                try_removing_snapshot(&self.pending_path);
            }
        } else {
            // should only be one or this is weird
            debug_assert!(self.snapshots.len() == 1);
            for snapshot in self.snapshots.iter() {
                match snapshot.op {
                    Operation::Accept | Operation::AcceptAll => {
                        try_removing_snapshot(&self.pending_path);

                        if let Some(ref old) = snapshot.old {
                            if let Some(path) = old.build_binary_path(&self.target_path) {
                                try_removing_snapshot(&path);
                            }
                        }

                        if let Some(path) = snapshot.new.build_binary_path(&self.pending_path) {
                            try_removing_snapshot(&path);
                        }

                        // We save at the end because we might write a binary file into the same
                        // path again.
                        snapshot.new.save(&self.target_path)?;
                    }
                    Operation::Reject | Operation::RejectAll => {
                        try_removing_snapshot(&self.pending_path);

                        if let Some(path) = snapshot.new.build_binary_path(&self.pending_path) {
                            try_removing_snapshot(&path);
                        }
                    }
                    Operation::Skip | Operation::SkipAll => {}
                }
            }
        }
        Ok(())
    }
}
