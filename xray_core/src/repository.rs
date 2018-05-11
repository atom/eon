use buffer;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::rc::Rc;
use uuid::Uuid;

type ReplicaId = Uuid;
type LamportTimestamp = usize;
type LocalTimestamp = usize;
type LocalEpochId = usize;
type NodeId = OperationId;

struct CommitSha {
    bytes: [u8; 20],
}

type VectorClock = HashMap<ReplicaId, LocalTimestamp>;

// A unique state in the repository is defined by an epoch and a state vector relative to the start
// of that epoch. The timestamp is assumed to be 0 for any replica id that does not have an entry
// in the map, which indicates it did not produce any operations in that epoch.
type Version = (EpochId, VectorClock);

// An Xray repository can be thought of in loose analogy to a git repository. It records a
// fine-grained conflict-free edit history for one or more work trees. An Xray repository can
// optionally map itself to a Git repository, and at least at first the only way to share changes
// between two work trees will be via Git.
struct Repository {
    replica_id: ReplicaId,
    work_trees: HashMap<WorkTreeId, WorkTree>,
    nodes: HashMap<NodeId, Rc<RefCell<Node>>>,
    epochs: HashMap<EpochId, Epoch>,
    local_clock: LocalTimestamp,
}

// A work tree contains a single linear development history and can be shared by multiple replicas.
// Work trees may or may not be mirrored to a local directory on the file system, depending on the
// use case. In-memory-only work trees will be used by remote workspaces, which will only fetch the
// names of all files and omit file contents in order to make joining a remote workspace fast.
// Mirrored work trees will be used by local workspaces, and will also be useful when
// interoperating with external tools that need access to the state of the work tree but aren't
// designed with Xray in mind.
struct WorkTree {
    version: Version,
    root: Node,
    epochs: HashMap<EpochId, Epoch>,
    local_clock: LocalTimestamp,
}

// Any replica can create a new work tree and assign it a unique id based on the `local_clock` of
// the `Repository` replica.
struct WorkTreeId {
    replica_id: ReplicaId,
    timestamp: LocalTimestamp,
}

// When an Xray repository is also a Git repository, history can become non-linear due to braching.
// CRDTs allow for concurrent editing, but they assume that the operations converge on a single
// history. To map the fine-grained but linear history of CRDTs to the coarse-grained but branching
// history of Git, we introduce epochs.
//
// For now, an Xray repository that does not map to Git will only ever contain one initial epoch.
// When Git is involved, we create a new epoch every time the `HEAD` of the underlying Git
// repository changes. Conceptually, an epoch contains a fine-grained linear history of all changes
// that occur until the next commit or other movement of `HEAD`.
//
// Creating a new epoch closes the previous epoch at whatever state the local replica happens to be
// in at the time the new epoch is created, assigning the parent's `end_state`. Operations
// concurrent to the creation of a new epoch are automatically cancelled due to the fact that they
// fall after the point in time at which the previous epoch was ended. If two epochs are created
// concurrently, which is detected when an epoch creation occurs in anything other than the most
// recent epoch, we break the tie based on the replica id of the creator, reassign the parent's
// `end_state` and discard the other concurrently-created epochs.
struct Epoch {
    local_id: LocalEpochId,
    parent_id: EpochId,
    end_version: Option<VectorClock>,
    commit_sha: Option<CommitSha>,
    local_clock: LocalTimestamp, // This clock is used to generate timestamps for OperationIds
}

// Any replica can create a new epoch, basing its id on the `local_lock` of the current `WorkTree`.
#[derive(PartialEq, Eq)]
struct EpochId {
    replica_id: ReplicaId,
    timestamp: LocalTimestamp,
}

// An operation represents some kind of modification to a work tree. This includes creating,
// moving, and deleting files, as well as modifying the contents of a file. It also includes the
// creation of a new epoch, which is kind of a meta-operation.
struct Operation {
    id: OperationId,
    payload: OperationPayload,
}

enum OperationPayload {
    CreateEpoch {
        id: EpochId,
        epoch: Epoch,
    },
    MoveNode {
        node_id: NodeId,
        new_parent_id: NodeId,
        lamport_timestamp: LamportTimestamp,
    },
    RenameNode {
        node_id: NodeId,
        name: String,
        lamport_timestamp: LamportTimestamp,
    },
    DeleteNode {
        node_id: NodeId,
    },
    UpdateTextNode {
        node_id: NodeId,
        operation: buffer::Operation,
    },
    UpdateBinaryNode {
        node_id: NodeId,
        lamport_timestamp: LamportTimestamp,
    },
}

// All operations need an id, which describes the tree to which the operation should be applied,
// the replica that generated the operation, as well as a specific point in logical time on the
// replica generating the operation, defined by an `epoch_id` and a `timestamp` relative to the
// start of that epoch.
#[derive(PartialEq, Eq)]
struct OperationId {
    epoch_id: EpochId,
    replica_id: ReplicaId,
    timestamp: LocalTimestamp,
}

struct Register<T> {
    // RegisterEntry values form a total ordering and the first entry that is compatible with a
    // given version is considered to contain the value of the register for that state.
    // Eventually we can persist older entries to the database and only keep a small subset in
    // memory.
    entries: BTreeSet<RegisterEntry<T>>,
}

// The registry entry associates the desired value with an operation id, which identifies the
// replica that produced the value and the local time it was produced, along with a lamport
// timestamp to enforce causal ordering. Entries should be sorted descending by lamport timestamp,
// and ties can be broken with the replica id of the entry's id.
struct RegisterEntry<T> {
    id: OperationId,
    lamport_timestamp: LamportTimestamp,
    value: T,
}

// A Node represents a file or directory. Its name and parent directory are expressed as registers,
// which allow the node to be renamed and moved concurrently and can be evaluated with respect to a
// specific Version.
struct Node {
    id: OperationId,
    name: Register<String>,
    alternate_name: Option<String>,
    parent_id: Register<NodeId>,
    content: NodeContent,
}

enum NodeContent {
    Directory {
        children: BTreeMap<String, Vec<Rc<RefCell<Node>>>>,
    },
    TextFile {
        contents: buffer::Buffer,
    },
    BinaryFile {
        bytes: Register<Vec<u8>>,
    },
}

impl<T> Register<T> {
    fn set(&mut self, id: OperationId, lamport_timestamp: LamportTimestamp, value: T) {
        self.entries.insert(RegisterEntry {
            id,
            lamport_timestamp,
            value,
        });
    }

    fn value(&self) -> Option<&T> {
        self.entries.iter().next_back().map(|entry| &entry.value)
    }
}

impl<T> PartialEq for RegisterEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for RegisterEntry<T> {}

impl<T> PartialOrd for RegisterEntry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for RegisterEntry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.lamport_timestamp.cmp(&other.lamport_timestamp) {
            Ordering::Equal => self.id.replica_id.cmp(&other.id.replica_id),
            result @ _ => result,
        }
    }
}
