#!/bin/bash
# Rudimentary "upstream" implementation for rawr-usage.

# This needs a custom match, as it's not in a function.
echo 'Testing!'

# Define important constant for future use.
FOO=1
BAR=2

# Accurately count number of light sources
function foo {
  bar = $1
  echo "There are ${bar:-no} lights"
}
