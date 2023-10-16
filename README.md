Leetasm Navigator
========
A policy gradient based model refinement tool.

## Build Instructions

Update `UEFI_PATH` environment variable to the appropriate tracer directory,
in this case:

```bash
export UEFI_PATH=/opt/leetasm/scratch/leetasm-mono/workbench
```

Build the target module with `bench.py`, in this case:

```bash
cd src/middleware/
python3 bench.py build CromulenceDemo1
```

Update (if required) the target module and choice of tracer script. This code is
in `navigator/envs/cannoli_streaming_client.py` and modifying the
tracer is a simple as tweaking the `run` command args. `CromulenceDemo1` should
match the target module as built above, and `nav` is the appropriate tracer
script in the `leetasm-uefi-testbench/zoo/{target module}` directory. The last
two arguments determine `EFI_VARS` (if necessary, if not then `None`) and a
vertical vs horizontal tmux split. As an example:

```python
self.bench = Bench()
self.bench.run("CromulenceDemo1", "nav", None, True)
```

## Run Instructions

Invoke NAV (or the learner of choice) which invokes `CannoliStreamingClient`
to proxy choices and responses. For full NAV, invoke from the bin
directory with `entry.py`. Standalone benchmark scripts are (currently) located
in the `demo` directory and should be invoked there. *Note, all NAV
invocations should currently be made from within a tmux shell, since the
tracer spawns a new tmux window to show the qemu-system-x86_64 output*. As an
example, see below. *Note: make the sure the tmux shell's UEFI_PATH environment
variable is updated per above*.

```bash
make run
```

Successful tests _should_ clean up processes nicely. But any failed test (due to
runtime errors or killing processes out of order), will leave zombie processes
running. To be safe, clean up the zombies by running `./kill.sh` in the bin
directory. This can be run from within the tmux shell. Output from this script
can be disregarded - it may try and kill processes that do not exist or that the
`leetasm` user does not have privilege over. That is ok. *Note: this script
should be run in a `leetasm` user terminal*.

```bash
make kill
```

Navigating in tmux can be tricky, the following are useful tips and tricks:

Jump to a new window:

```
[Ctrl + b, direction with arrow keys]
```

Kill a window:

```
[Ctrl + d]
```

Kill a window with qemu running in it:

```
[Ctrl + a, x] # kills qemu
[Ctrl + d]
```

Unlock scrolling to scroll up in the terminal:

```
[Ctrl + [] # open bracket
```
