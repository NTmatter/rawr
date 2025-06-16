# RAWR - Reimplement And Watch Revisions
Helper tool for porting large or changing codebases. Manually establish a mapping to the upstream codebase(s) to track completion and highlight when upstream changes require reimplementation.

RAWR leverages Tree-Sitter to track points of interest in the upstream codebase, and GitOxide to traverse the upstream repository while looking for changes in the observed code. Information about points of interest are stored as metadata and annotations in the downstream codebase, avoiding the need to bother or burden the upstream maintainers.


# Reference
* [Tree-Sitter](https://tree-sitter.github.io/): Extract representation of codebases.
  * [Playground](https://tree-sitter.github.io/tree-sitter/7-playground.html)
* [Byron/gitoxide](https://github.com/Byron/gitoxide): Traverse repositories to identify changes.
* [git-notes](https://git-scm.com/docs/git-notes): Add custom metadata and annotations to git commits.
