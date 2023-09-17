# ssedit - Structured Data Stream Editor
ssedit (***S***tructured Data ***S***tream ***Edit***or) is a stream editor for structured data formats that retains all original formatting and will retain original tabs/spaces/line ending formats. At the moment it only supports JSON but in the future it will be extended to support YAML amd INI.

## Usage
At the moment ssedit only supports input from STDIN.

Here is a simple example using JSON path to reference data in the sample.json file found in the root of this GitHub repo.

```
$ cat sample.json | ./target/debug/ssedit -q '$.batters.batter[1].type'
Chocolate%
$
```

Here is a simple example using JSON path to reference array data in the sample.json file found in the root of this GitHub repo. There is currently a bug with traditional JSON path root array referencing so there's been a slight change to the syntax.

```
$ cat sample2.json | ./target/debug/ssedit -q '$.[0].id'
5001%
$
```