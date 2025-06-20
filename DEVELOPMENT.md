# Development
As a goal, RAWR chunks and checksums upstream codebases and identifies changes with respect to annotations in a target codebase.

The project is currently in the early poc/experimental phase, with most effort being expended on learning to use the underlying libraries, building matchers and prototyping structure.

Current Task:
- Working on `bin/hello-scrape.rs`
- Enumerate and checksum all items of interest

Next Task:
- Persist items of interest
- Track movement of items of interest

Terminology:
- `Upstream Codebase`: The codebase being ported
- `Working Codebase`: The reimplementation

Binaries are being used for early prototyping:
- `hello-matches`: (DONE) Early exploration with tree-sitter.
- `interesting-items`: (DONE) More advanced matches with tree-sitter.
- `hello-git`: (DONE) Early exploration with the gitoxide library.
- `hello-topology`: (DONE) Traverse a series of commits with gitoxide.
  - Additional work is needed for topological sorting.
- `hello-toml`: (ON-HOLD) Early prototypes of configuration file.
  - Use hard-coded matches for now, and circle back after getting more experience with scrape.
  - Will be better-informed once some data structures have been built.
- `hello-scrape`: (WIP) Parse a hard-coded set of interesting items from an entire git repository.
  - What language and codebase should the scrape focus on?
    - Let's use Java for its structural simplicity, and run it against the SDFS codebase.
  - Need to record ignored items
- `hello-annotations`: (NEXT) Parse annotations in the working codebase.
  - Avoid playing with XML/YAML/TOML/JSON or alternative formats in comments for now.
  - Rust annotations only. Keep scope away from other languages for now.
- `hello-comparison`: (LATER) Identify items changed since recording in the working codebase.
  - What's the initial upstream codebase?
    - Use this to direct early language matchers.
    - Netatalk: 125kloc of C. Very interesting, under active development. Mixed GPLv2, MIT, others.
    - sdfs: 55kloc of Java. Project seems a bit dead, but is at least small. GPLv2.
    - glusterfs: 500kloc.
    - sqlite: 350kloc, mostly C. Too big for POC, and has already been done.
    - libtree-sitter: Core is about 18kloc of C and ~5kloc of JS. Looks MIT-licensed.
      - Requires matchers for C and JS, and shells out to Node.
        - Processing and transpiling the grammars could be problematic.
      - Might be the most viable based on size and license.
    - SDFS looks like more fun.
    - willscott/go-nfs: Less than 5kloc of golang, Apache2.
    - IRIS photosensitivity tester. Less than 10k lines of C++, and BSD-licensed. 
      - [Someone](https://www.reddit.com/r/rust/comments/1l6ypys/comment/mwur4rb/) wants a port of [electronicarts/IRIS](https://github.com/electronicarts/IRIS).
      - Depends on OpenCV. There are Rust bindings.
    - OpenCV: 500k-1M lines of C/C++. Apache2 license.
      - Could be an interesting scaling test.
    - How about something smallish in Python or C#?

# Notes
Gather checksums of functions, classes, structs, and files with and without whitespace and comments. The character and byte offsets might also be useful for extracting checksums. Detecting changes inside comments vs implementation could be interesting. This will likely involve a lot of per-language work, as the tree-sitter grammars do not seem to have normalized names for functions, methods, classes, and other objects of interest.

I could opt for a full-fledged diffing library like [mitsuhiko/similar](https://github.com/mitsuhiko/similar), [pijul/diffs](https://nest.pijul.com/pijul/diffs), or [pascalkuthe/imara-diff](https://github.com/pascalkuthe/imara-diff), but I suspect it might be easier to subtract a pair of sets using `(identifier, hash) -> Tree-Sitter Data` and a custom comparator.

Identifying the location of changes could be extremely expensive depending on the size of the codebase. Each watch will require traversing the upstream history starting at the last recorded commit. This can likely be batched and parallelized to avoid a quadratic number of parses (watched items * revisions to check). How difficult is it to topologically sort commits? Does Gix have anything for querying inclusion in the commit graph?
* Read configuration and annotations from downstream.
* Build topologically-sorted structure for lookup.
* Start after minimum commit referenced by annotations, walk forward, and only try to parse/checksum if we are after the annotation's commit.

It should be possible to use Tree-Sitter to update downstream annotations to minimize time spent searching for changes.

For early implementation, only enumerate the annotations.

Build rudimentary HTML support with XML in attributes. Use this to build documentations from an upstream.
An XML Schema is the way to go, allowing for direct integration with anything that is written with or operates on XML.
Some languages will still need comment parsing, but nested languages are a problem for later.

# Reference
* [Tree-Sitter](https://tree-sitter.github.io/): Extract representation of codebases.
* [Byron/gitoxide](https://github.com/Byron/gitoxide): Traverse repositories to identify changes.
* [git-notes](https://git-scm.com/docs/git-notes): Add custom metadata and annotations to git commits.
