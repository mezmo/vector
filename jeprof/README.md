## Profiling with `jemalloc`

### Enable profiling

First, `jemallocator` needs to be built with the `profiling` feature flag. It's likely that
our upstream merge process removed the flag, or we have done so ourselves for good hygiene.
Edit `Cargo.toml` to look something like this:

```toml
tikv-jemallocator = { version = "0.5.4", default-features = false, features = ["profiling"], optional = true }
```

**Note:** As of 2025, the use of `unprefixed_malloc_on_supported_platforms` feature appears to
NOT work, so it should be removed when doing profiling.

Next, profiling needs to be enabled on the k8s Vector cluster. To do that
you'll edit the `vector` configmap by adding a `malloc-conf` entry:


```yaml
data:
  # ...
  malloc-conf: prof:true,lg_prof_interval:37 # Creates a heap dump every 128 GiBs (2^37) of allocations
```

Save the configmap and restart the vector statefulset for it to take effect.
Once it does take effect you'll start to see `*.heap` files show up in the root
of the Vector pod's file system.


*Important*: Remember to disable profiling once when done profiling; otherwise,
it could fill up the root file system.

### Analyzing heap dumps

Use `kubectx` to switch to desired k8s context. Then you'll run:

```sh
$ ./report.sh <pod name> --text
```

This will generate a "text" report to stdout. It will also download a
collection of the current `*.heap` files in a tarball:

```sh
$ ./report.sh vector-gen0-1 --text
...
$ ls
heap-pipeline-vector-gen0-1-2023-03-22-16-26-54.tar.gz
...
```

This heap tarball file can be provided directly to `report.sh`:

```sh
$ ./report.sh heap-pipeline-vector-gen0-1-2023-03-22-16-26-54.tar.gz v0.27.0.6
```

This allows reports from previous dumps to be regenerated anytime.

Here's a description for what the columns of `--text` reports mean
([reference]):

```
The first column contains the direct memory use in MB.  The fourth column
contains memory use by the procedure and all of its callees.  The second and
fifth columns are just percentage representations of the numbers in the first
and fourth columns.  The third column is a cumulative sum of the second column
(i.e., the kth entry in the third column is the sum of the first k entries in
the second column.)
```

`--text` is not the only type of report that can be made by `jeprof` consult
its documentation for more information.

[reference]: https://www.igorkromin.net/index.php/2018/06/07/post-processing-jemalloc-jeprof-heap-dump-files-for-statistical-analysis/
