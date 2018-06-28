# Eon

## Development process

### Experiment

At this phase, this code is focused on learning. Whatever code we write should be production-quality, but we don't need to support everything at this phase. We can defer features that don't contribute substantially to learning.

### Documentation-driven development

Before coding, we ask ourselves whether the code we're writing can be motivated by something that's written in the guide. The right approach here will always be a judgment call, but let's err on the side of transparency and see what happens.

### Disciplined monorepo

All code related to Xray should live in this repository, but intra-repository dependencies should be expressed in a disciplined way to ensure that a one-line docs change doesn't require us to rebuild the world. Builds should be finger-printed on a per-component basis and we should aim to keep components granular.

### Community SLA

Well-formulated PRs and issues will receive some form of response by the end of the next business day. If this interferes with our ability to learn, we'll revisit.

## Contributing

Interested in helping out? Welcome! Check out the [CONTRIBUTING](./CONTRIBUTING.md) guide to get started.

## Tasks
* [x] Update index from a depth-first traversal of an external file system
* [ ] Mirror remote operations to the local file system
* [ ] File system changes that happen in the middle of a scan
  * For example, if we scan a directory and then that same directory is moved later in the depth-first traversal before the scan completes, we would scan it again.
* [ ] Applying remote operations to an index that doesn't match the state of the file system
  * For example, a remote user adds a directory "b" inside a directory "a", but directory "a" is renamed to "c" before we can apply the result of the operation.
* [ ] Watch the file system
* [ ] Scan the file system
