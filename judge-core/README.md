# Judge Core

Or we should call it 'Judge Utils'.
This libray will provide various method, which will be needed in online-judge system.

## Overview

See what we've got (or plan to do) currently:

- a **compiler** which build target from given src to some place
- a **sandbox** mainly based on `rlimit` and `seccomp`, helps you to spawn process safely
- a **monitor** (or judger) with sandboxes,
enables you to run single part of judge test_case (if you got everything needed for judge)
- a **judge_builder** (WIP) to provide a higher level interface to start a judge,
supposing the judge directory structure is following [ICPC Problem Package format](https://icpc.io/problem-package-format/examples/directory_structure)

## Use strace for seccomp withlist dev

Currently we are using a very normal approach by just running the following command:

``` shell
strace ./tmp/icpc/program
```

It will be better if we have a strace runned executor.
