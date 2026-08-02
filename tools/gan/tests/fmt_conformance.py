"""GEP-0016 conformance for `gan fmt`.

Covers: the fixture reformat (R003/R004/R005/R008), idempotency,
`--check` exit codes, heredoc value preservation, and the R006
verification refusing a corrupted rewrite.

Usage: python fmt_conformance.py <gan-executable> [<venv-python>]
"""

import os
import shutil
import subprocess
import sys
import tempfile

GAN = os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else "gan"
PY = os.path.abspath(sys.argv[2]) if len(sys.argv) > 2 else None
HERE = os.path.dirname(os.path.abspath(__file__))

failures = []


def check(cond, label):
    print(("PASS " if cond else "FAIL ") + label)
    if not cond:
        failures.append(label)


def run(args, cwd):
    return subprocess.run([GAN] + args, cwd=cwd, capture_output=True, text=True)


with tempfile.TemporaryDirectory() as tmp:
    src = os.path.join(tmp, "src")
    os.makedirs(src)
    shutil.copy(os.path.join(HERE, "fmt", "messy.gan.in"), os.path.join(src, "m.gan"))

    # --check on a messy tree: exit 1, no writes
    before = open(os.path.join(src, "m.gan")).read()
    r = run(["fmt", "--check", "src"], tmp)
    check(r.returncode == 1, f"--check exits 1 on unformatted tree (got {r.returncode})")
    check(open(os.path.join(src, "m.gan")).read() == before, "--check writes nothing")

    # formatting matches the canonical fixture
    r = run(["fmt", "src"], tmp)
    check(r.returncode == 0, f"fmt exits 0 ({r.stderr[:200]})")
    got = open(os.path.join(src, "m.gan")).read()
    want = open(os.path.join(HERE, "fmt", "canonical.gan")).read()
    check(got == want, "fixture formats to the canonical form")
    check("&($math.sqrt/1)" in got, "R008 capture parenthesization applied")

    # heredoc value: dedented content is untouched
    check("  docs stay put\n" in got, "heredoc body shifted as a unit (R005)")

    # idempotency
    r = run(["fmt", "src"], tmp)
    check("0 file(s) reformatted" in r.stdout, "fmt is idempotent (R006)")
    r = run(["fmt", "--check", "src"], tmp)
    check(r.returncode == 0, "--check exits 0 on a formatted tree")

# R006: a corrupted rewrite is refused (white-box via the compiled module)
if PY:
    code = (
        "import gandora_tool.fmt as f\n"
        "ok = f._verify('x = 1  # keep\\n', 'x = 1  # keep\\n')\n"
        "bad_comment = f._verify('x = 1  # keep\\n', 'x = 1  # changed\\n')\n"
        "bad_term = f._verify('x = 1\\n', 'x = 2\\n')\n"
        "print(ok == 'ok' and bad_comment != 'ok' and bad_term != 'ok')\n"
    )
    r = subprocess.run([PY, "-c", code], capture_output=True, text=True)
    check(r.stdout.strip() == "True", f"R006 verification refuses corrupted rewrites ({r.stdout.strip()} {r.stderr[:200]})")

print("=" * 40)
print("ALL PASS" if not failures else f"{len(failures)} FAILURES: {failures}")
sys.exit(1 if failures else 0)
