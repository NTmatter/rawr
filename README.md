# RAWR - Reimplement And Watch Revisions
Helper tool for porting large or changing codebases. Manually establish a mapping to the upstream codebase(s) to track completion and highlight when upstream changes require reimplementation.

RAWR leverages Tree-Sitter to track points of interest in the upstream codebase, and GitOxide to traverse the upstream repository while looking for changes in the observed code. Information about points of interest are stored as metadata and annotations in the downstream codebase, avoiding the need to bother or burden the upstream maintainers.

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
