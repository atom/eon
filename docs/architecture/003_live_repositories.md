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

