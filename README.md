
# workspace-linker

workspace-linker is a tool to create links between your workspaces.

Links are defined in package.json of each workspace and root package.json. Each package.json can specify its own local links and workspaces from which links will be imported. Imports are specified by its path relative to root package.json.

By default links will be linked to the same directory where the link is specified or imported (can be skipped with -w), and also to node_modules (can be skipped with -n).

Example:

```
{
  "name": "ws1",
  "version": "1.0.0",
  "description": "",
  "main": "index.js",
  "workspaceLinks": {
    "local": {
      "@/ws1Src": "./src"
    },
  }
}
```
```
{
  "name": "ws2",
  "version": "1.0.0",
  "description": "",
  "main": "index.js",
  "workspaceLinks": {
    "local": {
      "@/ws2Src": "./src"
    },
    "imports": [
      "ws/ws1"
    ]
  }
}
```

This will create directory structure in ws2 workspace like
```
./package.json
./src
./@/ws1Src -> ../ws1/src
./@/ws2Src -> ./src
./node_modules/@/ws1Src -> ../../ws1/src
./node_modules/@/ws2Src -> ../src
```
