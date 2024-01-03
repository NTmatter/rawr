# RAWR - Reimplement And Watch Revisions
Helper tool for porting large or changing codebases. Manually establish a mapping to the upstream codebase(s) to track completion and highlight when upstream changes require reimpleementation.

RAWR leverages Tree-Sitter for parsing downstream annotations and identifying features in the upstream codebase. Annotations and other metadata are stored in the downstream repository, avoiding the need to bother or burden the upstream maintainers.

# Notes
Gather checksums of functions, classes, structs, and files with and without whitespace and comments. The character and byte offsets might also be useful for extracting checksums. Detecting changes inside comments vs implementation could be interesting.

I could opt for a full-fledged diffing library like [mitsuhiko/similar](https://github.com/mitsuhiko/similar), [pijul/diffs](https://nest.pijul.com/pijul/diffs), or [pascalkuthe/imara-diff](https://github.com/pascalkuthe/imara-diff), but I suspect it might be easier to subtract a pair of sets using `(identifier, hash) -> Tree-Sitter Data` and a custom comparator.

# Reference
* [Tree-Sitter](https://tree-sitter.github.io/): Extract representation of codebases.
* [Byron/gitoxide](https://github.com/Byron/gitoxide): Traverse repositories to identify changes.
* [git-notes](https://git-scm.com/docs/git-notes): Add custom metadata and annotations to git commits.
