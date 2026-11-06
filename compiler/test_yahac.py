#!/usr/bin/env python3
"""Differential tests: yahac-compiled C versus the reference interpreter.

Usage: test_yahac.py [--yahac CMD] — CMD defaults to running yahac.yaha on
the reference interpreter. Pass a native yahac binary to test the bootstrap.
"""

import argparse
import os
import pathlib
import shlex
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
INTERP = ROOT / "target" / "release" / "yahalang"
YAHAC = ROOT / "compiler" / "yahac.yaha"

EXAMPLES = [
    ("hello_world.yaha", b""),
    ("add_two_numbers.yaha", b"20\n22\n"),
    ("integer_echo.yaha", b"-17\n"),
    ("array_access.yaha", b""),
    ("array_copy.yaha", b""),
    ("array_push_pop.yaha", b""),
    ("countdown.yaha", b""),
    ("truth_machine.yaha", b"0\n"),
    ("character_echo.yaha", "héllo, 世界\n\0🦀".encode()),
    ("reverse_input.yaha", "abc가나\n".encode()),
    ("break_on_zero.yaha", b"3\n2\n0\n"),
    ("break_on_zero.yaha", b"3\n2\n"),
    ("string_echo.yaha", "가나다\n".encode()),
    ("string_length.yaha", "가나🦀\r\n".encode()),
    ("print_character_array.yaha", "Hello, 안녕!\n".encode()),
    ("compare.yaha", b"3\n5\n"),
]

N = lambda n: "hayaha " + "ppuru" * n + " yaha"  # noqa: E731

# (name, source, stdin, lang)
CASES = [
    ("arith", f"{N(7)} {N(2)} ura yaha ; iiiiyaha ppururu {N(7)} {N(2)} ura hat ; iiiiyaha ppururu"
              f" {N(7)} {N(2)} ura hathat iiiiyaha ppururu {N(7)} {N(3)} hat iiiiyaha ppururu", b""),
    ("neg-div", f"{N(7)} ura yat {N(2)} ura hat ; iiiiyaha ppururu {N(7)} ura yat {N(2)} ura hathat"
                " iiiiyaha ppururu", b""),
    ("rem-then-mul", f"{N(7)} {N(3)} {N(2)} ura hathathat iiiiyaha ppururu", b""),
    ("compare", f"{N(1)} {N(2)} haa iya iiiiyaha ppururu {N(1)} {N(2)} haa yaha iiiiyaha ppururu"
                f" {N(2)} {N(2)} haa hat iiiiyaha ppururu", b""),
    ("swap-discard", f"{N(1)} {N(2)} iya ppuru iiiiyaha ppururu iiiiyaha ppururu {N(5)} ura ppuru", b""),
    ("yahayaha-is-two-adds", f"{N(1)} {N(2)} {N(3)} yahayaha iiiiyaha ppururu", b""),
    ("if-taken", f"{N(1)} iya yahayaha {N(4)} iiiiyaha ppururu ura yahayaha", b""),
    ("if-skipped", f"{N(0)} iya yahayaha {N(4)} iiiiyaha ppururu ura yahayaha {N(9)} iiiiyaha ppururu", b""),
    ("nested-break", f"{N(2)} iya yadayada {N(3)} iya yadayada ura iya ura yadayada iiiiyaha ppururu"
                     f" {N(1)} ura yaha ; ura yadayada iiiiyaha ppururu", b""),
    ("break-in-if", f"{N(5)} iya yadayada ppuru {N(2)} haa hat iya yahayaha ura iya ura yahayaha"
                    f" {N(1)} ura yaha ; ura yadayada iiiiyaha ppururu", b""),
    ("halt", f"{N(1)} iiiiyaha ppururu yat {N(2)} iiiiyaha ppururu", b""),
    ("halt-in-loop", f"{N(1)} iya yadayada {N(6)} iiiiyaha ppururu yat ura yadayada", b""),
    ("array-literal", f"[{N(0)}, {N(1)}, {N(2)}] ppuru # iiiiyaha ppururu ppuru {N(1)} @ iiiiyaha ppururu"
                      f" ppuru {N(1)} {N(9)} = {N(1)} @ iiiiyaha ppururu", b""),
    ("array-literal-shared", f"[{N(4)}] ppuru {N(0)} {N(8)} = {N(0)} @ iiiiyaha ppururu", b""),
    ("empty-literal", "[] # iiiiyaha ppururu [ ] yada ppururu iiiiyaha ppururu", b""),
    ("array-ops", f"yada hayaha {N(65)} yada ppuru {N(66)} yada ppuru yada urara {N(67)} yada ppuru"
                  " ppuru yada iiiiyaha yada ppururu iiiiyaha ppururu ppuru yada iiiiyaha", b""),
    ("array-get-set", f"yada hayaha {N(3)} yada ppuru ppuru {N(0)} {N(7)} yada hat {N(0)} yada iya"
                      " iiiiyaha ppururu", b""),
    ("pop", f"yada hayaha {N(1)} yada ppuru {N(2)} yada ppuru yada ura iiiiyaha ppururu yada ura"
            " iiiiyaha ppururu", b""),
    ("read-int-trim", "iiiiyaha iiiiyaha yaha iiiiyaha ppururu", b"  +12 \t\r\n-2\n"),
    ("read-int-min", "iiiiyaha iiiiyaha ppururu", b"-9223372036854775808\n"),
    ("read-char-eof", "urara iiiiyaha iiiiyaha ppururu urara iiiiyaha iiiiyaha ppururu", b"A"),
    ("read-line-empty", "urara yada yada ppururu iiiiyaha ppururu urara yada yada iiiiyaha", b"\nxy"),
    ("read-line-cr", "urara yada yada ppururu iiiiyaha ppururu", b"a\rb\r\n"),
    ("print-unicode", f"{N(54616)} iiiiyaha urara {N(10)} iiiiyaha urara", b""),
    ("semicolons", f";;{N(1)};{N(2)};yaha;iiiiyaha;ppururu;", b""),
    ("crlf", f"{N(1)}\r\n{N(2)}\r\nyaha iiiiyaha ppururu\r\n", b""),
    ("empty", "", b""),
    ("only-space", " \n\t\n", b""),
    ("korean", "하야하 뿌루뿌루 야하 이이이야하 뿌루루", b"", "ko"),
    ("japanese", "ハヤハ プルプルプル ヤハ ハヤハ プル ヤハ ウラ ヤハ ; イイイヤハ プルル", b"", "ja"),
    ("deep-128", "hayaha ppuru yaha " * 128 + "iya yahayaha " * 128 + N(3) + " iiiiyaha ppururu "
                 + "ura yahayaha " * 128, b""),
    # Runtime errors.
    ("err-underflow", "yaha", b""),
    ("err-div0", f"{N(1)} iiiiyaha ppururu {N(1)} {N(0)} ura hat", b""),
    ("err-overflow", f"{N(1)} " + "ppuru yaha " * 63 + "iiiiyaha ppururu", b""),
    ("err-type", "yada hayaha iiiiyaha ppururu", b""),
    ("err-index", f"[{N(1)}] {N(1)} @", b""),
    ("err-pop-empty", "yada hayaha yada ura", b""),
    ("err-int-input", "iiiiyaha", b"12x\n"),
    ("err-int-eof", "iiiiyaha", b""),
    ("err-int-overflow", "iiiiyaha", b"9223372036854775808\n"),
    ("err-char-utf8", "urara iiiiyaha", b"\xff"),
    ("err-line-eof", "urara yada", b""),
    ("err-surrogate", f"{N(55296)} iiiiyaha urara", b""),
    ("err-while-array", "yada hayaha iya yadayada ura yadayada", b""),
    # Compile errors.
    ("cerr-unknown", "hello", b""),
    ("cerr-hyphen", "ura-yaha", b""),
    ("cerr-cr", f"{N(1)}\r", b""),
    ("cerr-unclosed-number", "hayaha ppuru", b""),
    ("cerr-unclosed-block", f"{N(1)} iya yahayaha", b""),
    ("cerr-mismatched-end", f"{N(1)} iya yahayaha ura yadayada", b""),
    ("cerr-stray-end", "ura yahayaha", b""),
    ("cerr-break-outside", f"{N(1)} iya yahayaha ura iya ura yahayaha", b""),
    ("cerr-lone-ura", "ura", b""),
    ("cerr-ura-ura", "ura ura", b""),
    ("cerr-lone-haa", "haa", b""),
    ("cerr-yada", "yada yaha", b""),
    ("cerr-urara", "urara", b""),
    ("cerr-ppururu", "ppururu", b""),
    ("cerr-iya", "iya", b""),
    ("cerr-iya-mixed", "iya yada yaha", b""),
    ("cerr-trailing-comma", f"[{N(1)},]", b""),
    ("cerr-literal", "[yaha]", b""),
    ("cerr-depth-129", "iya yahayaha " * 129 + "ura yahayaha " * 129, b""),
    ("cerr-mixed-language", "hayaha 야하", b"", "en"),
]


def run(cmd, stdin, **kw):
    return subprocess.run(cmd, input=stdin, capture_output=True, timeout=600, **kw)


def reaction(stderr):
    """The reference prints 'path: reaction'; compiled programs print 'reaction'."""
    text = stderr.decode(errors="replace").strip()
    return text.rsplit(": ", 1)[-1] if text else ""


def compile_c(yahac, source, tmp, name):
    result = run(yahac, source)
    if result.returncode != 0:
        return None, result
    c_file = tmp / f"{name}.c"
    c_file.write_bytes(result.stdout)
    exe = tmp / name
    subprocess.run(["gcc", "-O1", "-w", "-o", str(exe), str(c_file)], check=True)
    return exe, result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--yahac", help="native yahac command (default: interpreted yahac.yaha)")
    args = parser.parse_args()
    yahac = shlex.split(args.yahac) if args.yahac else [str(INTERP), "run", str(YAHAC), "--lang", "ko"]

    cases = [(f"example:{f}", (ROOT / "examples" / f).read_bytes(), i, "ko") for f, i in EXAMPLES]
    cases += [(c[0], c[1].encode(), c[2], c[3] if len(c) > 3 else "en") for c in CASES]

    failures = 0
    with tempfile.TemporaryDirectory() as tmp:
        tmp = pathlib.Path(tmp)
        for index, (name, source, stdin, lang) in enumerate(cases):
            src = tmp / f"case{index}.yaha"
            src.write_bytes(source)
            ref = run([str(INTERP), "run", str(src), "--lang", lang], stdin)
            exe, comp = compile_c(yahac, source, tmp, f"case{index}")
            if exe is None:
                got = (comp.returncode, b"", "haa?")
            else:
                out = run([str(exe)], stdin)
                got = (out.returncode, out.stdout, reaction(out.stderr))
            want = (ref.returncode, ref.stdout, reaction(ref.stderr))
            if exe is None and ref.returncode != 0 and reaction(ref.stderr) == "haa?":
                want = (comp.returncode, b"", "haa?")  # both rejected at compile time
            ok = got == want
            failures += not ok
            print(f"{'ok  ' if ok else 'FAIL'} {name}")
            if not ok:
                print(f"     want {want}\n     got  {got}")
    print(f"{len(cases) - failures}/{len(cases)} passed")
    sys.exit(1 if failures else 0)


if __name__ == "__main__":
    main()
