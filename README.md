# Bxckup
###### Very simple file copy-backup

```config.toml```
```
[[task]]
# source directory
source = "B:\\bxckup\\test-source"
# target directory, will be created if it doesn't exist
target = "B:\\bxckup\\test-target"
# exclude files whose path contains any of the following strings, respects case
# matches at any point in path. make sure to use dir delmiters for your OS
exclude = [".git", "node_modules"]

[[task]]
...
```