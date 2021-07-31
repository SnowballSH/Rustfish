import os

if "RUSTFLAGS" not in os.environ:
    os.environ["RUSTFLAGS"] = ""

NATIVE = "-C target-cpu=native"

os.environ["RUSTFLAGS"] = ' '.join((NATIVE,))

print("NEW RUSTFLAGS: " + repr(os.environ["RUSTFLAGS"]))

command = "cargo build --release"

print("building with " + repr(command))

os.system(command)

os.environ["RUSTFLAGS"] = ""

print("RESTORED RUSTFLAGS TO " + repr(os.environ["RUSTFLAGS"]))
