# Epoch: Xray's real-time version control system

Xray ships with its own operation-based version control system called *Epoch*, which supports real-time, conflict-free synchronization and interoperates with Git. Epoch is enabled by default when editing via the Xray editor, and Epoch can also be integrated with other text editors via a shared-library or a standalone server that communicates over local sockets.

## Overview

When working with Git, you commit snapshots of your repository to a branch, then manually synchronize your copy of that branch with a remote replica by pulling and pushing commits. Epoch branches are similar to Git branches, but they are automatically persisted on every edit and are continuously synchronized across all replicas in real-time without requiring manual conflict resolution.

Real-time change synchronization means that instead of waiting for changes to be committed and pushed to GitHub, cloud-based services can interact with the state of a repository as the code is being actively being written. For example, a service like Code Climate could perform incremental analysis on every branch of an Epoch repository as it changes, inserting annotations into the repository that are be interpreted by client-side tooling. The ability for any replica to perform writes without risk of conflicts means that a cloud-based service could also perform edits such as code formatting.

Epoch branches persist every change as it occurs, allowing a specific moments in the editing history to be identified with a version vector. This allows developers to write code for extended sessions without committing, then scrub the history to identify relevant checkpoints in their work after the fact. These checkpoints could also be identified automatically via analysis of the edit history. Fine-grained versioning means that any state of the code can be deployed into a development environment or a staging server without the ceremony of a commit. Just make some edits and click "play" to try them out.

Epoch can be used as a standalone version control system, but it is also designed to interoperate smoothly with Git, meaning that an Epoch repository can also be a Git repository. Epoch branches are aware of the current Git branch, and Epoch automatically maps Git commit SHAs to Epoch version vectors as commits are created. If Epoch detects that the user has checked out a different Git commit, it automatically updates the Epoch branch to the appropriate version in the Epoch history and replicates the check-out to all the branch's replicas.

## Command line interface

When using Epoch via the Xray editor, command-line interaction should not be required, but covering it first will provide clarity to later sections of this document. All repository operations are performed through the `epoch` command.

* `epoch init` Create an Epoch repository in the current directory. Epoch will create a database file named `.epoch` in the current directory and add an entry for it to the `.gitignore`. It will then populate the database based on the current state of the file system and create a default `master` branch in which to store edits.

* `epoch remote add <name> <url>` Register a remote Epoch repository and allow the current repository to check out branches from that repository.

* `epoch sync [--force] [<remote>]` Synchronize the repository with the current state of the file system after disconnected operation. If this is a Git repository and the specified remote is reachable over the network, Epoch first attempts to fetch history mapping to the current Git `HEAD`. Once the fetch completes, Epoch history is synthesized for any Git commits that aren't present in the database. If the remote is unreachable, synchronization may fail, placing the current branch into *disconnected mode*, described later in the document.

* `epoch publish [<remote>] [<local-branch>[:<remote-branch>]]` Send the current state of a local branch to a remote replica, along with all future operations. If no `<local-branch>` is specified, it is assumed to be the current branch. If no `<remote>` is specified, it is assumed to be `origin`.

* `epoch unpublish [<remote>] [<local-branch>]` Stop sending operations for a local branch to a specific replica. If no `<local-branch>` is specified, it is assumed to be the current branch. If no `<remote>` is specified, it is assumed to be `origin`.

* `epoch checkout [<remote>/]<branch>` Start editing a branch. This fetches the branch's current state and subscribes to all future operations. Any edits will be broadcast to other replicas until `epoch unpublish` is run.

* `epoch checkout -b <branch>` Create a new branch based on the state of the current branch.

* `epoch reset <version>` Switch the current branch to the specified state in the repository and replicate this change out to other replicas.

* `epoch merge <branch>` Merge the specified branch into the current branch. This only succeeds if the current branch shares a common ancestor with the current branch.

## Xray integration

Xray is designed around Epoch, with the goal of being Epoch's ideal text editor implementation. When Xray opens a folder, it looks for an `.epoch` file at the folder's root. If one is detected, Xray automatically synchronizes the repository with `origin`. This synchronization is usually quick and often instant, but in rare circumstances it may take some time. If the user attempts to edit a file before synchronization is complete, they will be offered the option to cancel the synchronization and enter *disconnected mode*, which disables certain advanced features that require the full edit history. They can also check a box to *always* abort synchronization and skip the prompt the next time they attempt to edit prior to a complete synchronization.

Once synchronization is complete (usually in a less than few seconds), the user is free to make edits, and all changes are continuously persisted into the repository. The availability of a full edit history for every buffer enables recovery to the latest state in the event of power failure, infinite, non-chronological undo, stable permalinks to logical positions in the source code, and more. If you edit the same Epoch branch as another user, you will see their edits in real time as well as the location of their active cursor.

## Relationship to shared workspaces

Epoch repositories are designed to synchronize their state in real time to enable a variety of repository-centric use cases, but for the use case of multiple human developers collaboratively writing code, Xray *shared workspaces* may be a better fit.

While Epoch repositories are focused on real-time replication of code, Xray shared workspaces are focused on allowing multiple developers to share a full-fledged development environment on a single machine, which goes beyond the state of the source code. Guests can of course access any file that is reachable in the workspace, but they also have access to machine-specific resources such as a terminal and a debugger. Shared workspaces are also designed to be lightweight, so that in order to join a shared workspace, you can just click a link. You don't even need to have the workspace's code checked out.

The trade-off with shared workspaces is that they are centralized. To make workspaces quick to join, we replicate the minimum possible state to guests and perform many kinds of operations as remote procedure calls. If the host loses network connectivity, all collaboration in a shared workspace is terminated. However, if the host happened to be editing an Epoch repository and they were actively replicating changes to the cloud, one of the guests could easily check out that branch in their own workspace to continue editing. Xray could even prompt guests with this option automatically if we detect a host disconnect but the live repository they were editing is still reachable.

## Conceptual model

## Git Interop

## Synchronization

## Disconnected mode

---

Everything from here on is potentially out of date

## Live work trees

A Git repository is associated with one or more work trees, which contain the files and directories that the user actually edits before creating commits. We propose to augment Git with *replicated work trees*, which can be replicated across multiple machines and continuously synchronized in real time in a conflict-free fashion.

Live work trees are intended to support real-time collaborative coding and live streaming of edits to an audience. They also continuously persist all edits to disk without the need for an explicit save or commit step. Other than the presence of edits from other collaborators, using a live work tree in a supported editor feels exactly like using a traditional Git work tree. Files are opened, edited, and saved in the ordinary way, and you can interact with Git via any tool to commit, change branches, pull changes, etc.

All live work trees associated with a repository are persisted in a single database that sits alongside the Git index. Persisted operations are cross-referenced with commit SHAs, allowing the keystroke-level edit history behind any commit to be retrieved and replayed later. When necessary, Xray gracefully accounts for changes that occurred outside of Xray by synthesizing operations.

## CRDT representation of the file system

The file system CRDT is designed to interoperate with Git when it is available.

Files and directories are represented as nodes. When a *new* file or directory is created on a given replica, it is assigned a unique id based on the replica id and an incrementing sequence number. If a file or directory is modified that is already present in the Git repository but has not yet been referenced by the CRDT, its identifier is the SHA of the most recent commit and its current path. Since all replicas are assumed to have access to the same Git history, this ensures that pre-existing files and directories are not assigned duplicate identifiers. It also assures that all replicas associate files with the same initial contents based on their contents in the Git history.

Nodes are associated with last-writer-wins registers for their `name` and `parent_id`, which allow them to be concurrently renamed and relocated to different locations within the repository. Nodes also have a deleted flag (or set if we want to support undo). Nodes representing text files are associated with text CRDTs representing their contents.

Multiple nodes with the same name can potentially be assigned to the same directory due to concurrent modifications. In that case, only the entry with the lowest site id is shown and the rest are hidden. If we're worried about users losing their data, we could instead deterministically assign these conflicting files to new names like `foo.txt~` or support undo operations on the file tree.

## Synchronizing the CRDT with the file system

When Xray first starts or when it detects a change, it synchronizes its internal representation of the tree with the underlying file system.

If no Git repository is present or if `HEAD` has not changed since the last sync, it synchronizes changes via a diff. It attempts to detect moved files via a similarity strategy like to Git, and updates the `name` and `parent_id` of these files accordingly. New nodes are added for created files, and deleted flags are set for removed files.

If the underlying directory tree is associated with a Git repository, we store the most-recently-synced `HEAD` as a last-writer-wins register associated with the current working copy in the CRDT. If the `HEAD` has changed, either a new commit has been created or a different commit has been checked out.

A new commit has been created if the parent commit of the new `HEAD` matches the previous `HEAD` stored in our internal register.

* HEAD doesn't change
  * Generate operations based on a diff between previous snapshot and current snapshot of the file system
* HEAD changes
  * We detect the creation of a new commit
    * Determine the subset of existing operations that most closely map to the new commit and express it as a state bitmap
    * Apply any "fixup" operations required to transition from the operation subset and the state of the commit.
  * We detect the checkout of an existing commit


* Unified identifier space: Each replica maintains a single local clock
* Sparse state vectors: The state is a bitmap per site that represents a set of local clock values.


# Going further with the CRDT

CRDTs enable real-time collaborative editing, but they are also potentially rich sources of information about how a piece of data has changed over time. Properly leveraged, CRDTs could potentially form the basis of a new kind of ultra fine-grained, real-time-capable version control system that tracks changes at the keystroke level.

# Use cases

## Persistence without committing

Currently, the only way to persist code changes is to manually save them, make a commit, and push them to GitHub. For some users, who are used to workflows like those offered in Google Docs, explicitly "saving" a document is an alien concept. They expect their changes to be persisted automatically.

What if the entire edit history of every buffer in a repository were automatically persisted. You would never need to explicitly save. Committing would simply mark points of interest in this history, but you could also explore a keystroke-level history of the whole file or any region within the file going back to the moment of its creation.

## Live branches

Live branches extend the narrative of a Git branch to support real time collaboration. Currently, when two developers make changes to the same branch, they periodically synchronize their changes via `git pull` and `git push` and manually resolve merge conflicts. With live branches, any developer that checked out the same live branch would continuous broadcast changes and incorporate changes made by other developers.

Live branches are an alternative to the *shared workspace* model that is oriented around replication. With shared workspaces, there is conceptually only one replica that is shared by a host and visited by guests. With live branches, each participant is an equal peer and maintains their own replica, but changes are automatically synchronized among all peers that have checked out that branch.

It's unclear whether the live branch narrative is a good fit for collaborative coding. For asynchronous collaboration, it obviously makes sense for each collaborator to maintain their own replica, but when developers are coding together in real time, their workflow is more similar to that of a single developer. There are multiple people, but there is still only one logical stream of code that needs to reach coherence at specific moments to compile, run tests, etc. Shared workspaces allow a developer to take their traditional workflow, which involves writing code on a single machine, and enhance it by allowing other developers to participate. It's a simpler and more familiar narrative than multiple replicas which are synchronized in real time.

One benefit of each participant maintaining a replica is that collaboration can continue even if the host of the workspace goes offline. Live branches could also be a good pool for continuously pushing edits from a local repository to a server in the cloud, either for purposes of backup or to continuously share progress, which brings us to the next section.

## Live pull requests

Once we have live branches, a natural extension is live pull requests. These would be similar to traditional pull requests, but would always be updated in real time as developers made changes. This could provide a greater sense of presence and immediacy for distributed teams by letting developers "peek over the shoulders" of teammates who have opted to broadcast their work stream. Open source developers could also use live pull requests to live stream their work with a broader community, which could be a valuable educational resource if paired with an audio feed.

We could broadcast changes from a specific workspace without fully replicated live branches, but this would be much more limited. If the original creator of the live PR were not online, nobody else would be able to pick up and continue where they left off.

## Code replay

What if we tracked any piece of code in the repository back to the moment it was created? If we timestamped individual operations, we could correlate edits to a recorded audio stream of one or more developers collaboratively writing that code. Scrubbing through the history and listening in on the conversation as the code was being written might lead to insights about the system that couldn't be gained in any other way.

## Permanent links into source code

The buffer CRDT allows us to create *anchors* to a given position in the buffer that can always be resolved to the same logical position regardless of subsequent edits. Buffer CRDTs are not currently persisted however, so as soon as the buffer is closed, any anchors into its contents are invalidated. It  anchors to remain valid over the lifetime of the repository even in the face of file renames and subsequent edits.

We could potentially derive the state of an anchor from the Git history alone, so that an anchor would be defined as a SHA, a path, and an offset. This would do a reasonable job tracking file renames and could update the anchor perfectly in the presence of subsequent edits that did not directly overlap the anchor's position. For edits overlapping the anchor's line, we need to resort to character-level or syntactic diffs to preserve the anchor's position, which might work in many cases but wouldn't be 100% reliable depending on the nature of the diff.

The alternative is to extend our current approach and use the CRDT history instead. This would require us to persist a buffer's CRDT history for the lifetime of the repository. This approach slots in well with collaborative editing, which already requires CRDT-based anchors. Our first big use case for persistent anchors is linking messages in a discussion to source code, and we anticipate that source-linked discussions will be most valuable when they are paired with collaborative editing. It would be great if the same anchors we used during active editing of the buffer could be used at any point in the future to link back to the right point in the editing history.

## Live merge

When branching, developers could choose to automatically incorporate changes from the original branch as they occurred. Similarly, when merging a branch, developers could elect to incorporate all current and future edits on that branch in real time.

The actual value of such a feature is unclear. The primary purpose of branches in the Git model is to delay coordination between concurrent streams of change. Live merging of changes from another branch would necessarily *require* close coordination with the author of the upstream branch, which may defeat the purpose of branching to begin with.

The author of the upstream branch could defer coordination with the authors downstream branches, however, creating an asymmetry which might be useful in some settings. For example, imagine a classroom setting in which a teacher is writing tests for a function and then asks students to write the function. Each student could create a branch based on the teacher's branch and attempt to write the function. Then the teacher could continue to make edits, calling the function with different arguments and asking students to re-run tests to discover whether they covered different edge cases.

On a theoretical level, combining the fluidity of real-time collaboration with branching and merging and allowing changes to freely flow is intriguing. In practice however, it's unclear how this feature might be used.
