#!/usr/bin/env python3

"""
bench.py

Python script for building and running edk2 with vulnerable modules installed
"""

import os
import yaml
import argparse
import subprocess
import signal
from pathlib import Path
import shlex


class Bench:
    def __init__(self, debug=True):
        p = os.environ.get("UEFI_PATH")
        self.bench_dir = Path(p) if p else None
        self.gdb = None
        self.qemu = None
        self.debug = debug

    def run_bench_cmd(self, cmd, popen=False, **kwargs):
        fn = subprocess.Popen if popen else subprocess.run
        return fn(shlex.split(cmd), cwd=self.bench_dir, **kwargs)

    def clean(self):
        self.run_bench_cmd("bench clean")

    def build(self) -> int:
        self.run_bench_cmd("bench build")

    def run(self, pid):
        environ = {
            "BENCH_OVERRIDE_EXEC": "yes",
            "BENCH_RUN_NAV_PID": str(pid),
            "BENCH_RUN_GDB_CONTINUE": "yes",
            "BENCH_RUN_CONFIRM_QUIT_QEMU": "yes",
            "BENCH_RUN_CONFIRM_QUIT_GDB": "yes",
            "BENCH_RUN_CONFIRM_BUILD_MODULE_FIRMWARE": "no",
            "BENCH_VERBOSITY": "5",
            "BENCH_RUN_CREATE_QEMU_BASH_SCRIPT": "no",
            "BENCH_RUN_CREATE_GDB_BASH_SCRIPT": "no",
        }
        prof = os.environ.get("BENCH_PROFILE")
        if prof:
            environ["BENCH_PROFILE"] = prof
        runargs = {}
        if not self.debug:
            runargs["stdout"] = subprocess.DEVNULL
            runargs["stderr"] = subprocess.DEVNULL
        self.gdb = self.run_bench_cmd("bench run ", popen=True, env=os.environ | environ, **runargs)

    def read_from_pipe(self):
        pass

    def kill(self):
        if self.gdb and self.gdb.poll() is None:
            self.gdb.kill()
