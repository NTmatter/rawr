# RAWR - Reimplement And Watch Revisions
Helper tool for porting large or changing codebases. Manually establish a mapping to the upstream codebase(s) to track completion and highlight when upstream changes require reimpleementation.

RAWR leverages Tree-Sitter for parsing downstream annotations and identifying features in the upstream codebase. Annotations and other metadata are stored in the downstream repository, avoiding the need to bother or burden the upstream maintainers.

# Reference
* [Tree-Sitter](https://tree-sitter.github.io/): Extract representation of codebases.
* [Byron/gitoxide](https://github.com/Byron/gitoxide): Traverse repositories to identify changes.
* [git-notes](https://git-scm.com/docs/git-notes): Add custom metadata and annotations to git commits.
