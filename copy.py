import os
import sys

if sys.platform in ("win32", "cygwin"):
    os.system("cp ./target/release/rustfish.exe ./rustfish.exe")
else:
    os.system("cp ./target/release/rustfish ./rustfish")
