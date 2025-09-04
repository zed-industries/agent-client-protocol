# Ensure the src/ directory is on sys.path so tests can import the local 'acp' package
import os
import sys

SRC_DIR = os.path.join(os.path.dirname(__file__), "src")
if SRC_DIR not in sys.path:
    sys.path.insert(0, SRC_DIR)
