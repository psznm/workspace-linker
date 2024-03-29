
# High
* Add possibility to add workspaces root path, and make links to only single workspace. 
    Or just search for workspaces root if initial package.json has import but not workspaces. 

# Normal
* Check if same link names have conflicting paths. If yes, error.
* Check if there is non-existent import. Do not panic, propagate error.
* If package.json can not be parsed, fail with specification which file failed to parse. This can probably be panic?
* Collect all errors and write them out
* Improve code structure. It currently is a mess
* Ensure dir only once per dir
* Add option to print resolved links, instead of actually linking. Json or readable
* Maybe try some testing? Maybeeeeee? But lets not get crazy here
