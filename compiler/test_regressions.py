import itertools
import os
import pathlib
import signal
import subprocess
import tempfile
import unittest

from test_yahac import compare_results

ROOT = pathlib.Path(__file__).resolve().parent.parent


def process(code, stdout=b"", stderr=b""):
    return subprocess.CompletedProcess([], code, stdout, stderr)


def run_with_closed_stdout(executable, restore_signals=True, stdin=b""):
    read_end, write_end = os.pipe()
    os.close(read_end)
    try:
        return subprocess.run(
            [str(executable)], input=stdin, stdout=write_end,
            stderr=subprocess.PIPE, restore_signals=restore_signals, timeout=5,
        )
    finally:
        os.close(write_end)


def run_with_pending_input(executable):
    input_read, input_write = os.pipe()
    output_read, output_write = os.pipe()
    os.close(output_read)
    os.write(input_write, b"7\n")
    os.close(input_write)
    try:
        result = subprocess.run(
            [str(executable)], stdin=input_read, stdout=output_write,
            stderr=subprocess.PIPE, timeout=5,
        )
        return result, os.read(input_read, 64)
    finally:
        os.close(input_read)
        os.close(output_write)


class ComparisonResults(unittest.TestCase):
    def setUp(self):
        self.reference_error = process(1, stderr=b"source.yaha: 1:1: haa?\n")
        self.compiler_error = process(1, stderr=b"yahac.yaha: haa? haa?\n")

    def test_expected_compile_error_is_accepted(self):
        want, got = compare_results(self.reference_error, self.compiler_error, None)
        self.assertEqual(want, got)

    def test_unexpected_failure_statuses_are_rejected(self):
        for code in [2, 99, -15]:
            result = process(code, stderr=b"haa? haa?\n")
            want, got = compare_results(self.reference_error, result, None)
            self.assertNotEqual(want, got, code)

    def test_missing_or_wrong_diagnostics_are_rejected(self):
        for stderr in [b"", b"ui\n", b"different error\n", b"haa?\n"]:
            want, got = compare_results(self.reference_error, process(1, stderr=stderr), None)
            self.assertNotEqual(want, got, stderr)

    def test_partial_compiler_output_is_rejected(self):
        result = process(1, stdout=b"partial C", stderr=b"haa? haa?\n")
        want, got = compare_results(self.reference_error, result, None)
        self.assertNotEqual(want, got)

    def test_accepted_invalid_source_is_rejected(self):
        want, got = compare_results(self.reference_error, process(0, stdout=b"C"), None)
        self.assertNotEqual(want, got)

    def test_valid_source_must_compile_even_if_it_fails_at_runtime(self):
        reference = process(1, stderr=b"source.yaha: haa? haa?\n")
        want, got = compare_results(reference, self.compiler_error, None)
        self.assertNotEqual(want, got)

    def test_success_and_runtime_errors_compare_real_results(self):
        for reference, execution in [
            (process(0, stdout=b"42"), process(0, stdout=b"42")),
            (process(1, stderr=b"source: ui\n"), process(1, stderr=b"ui\n")),
        ]:
            want, got = compare_results(reference, process(0, stdout=b"C"), execution)
            self.assertEqual(want, got)


class RuntimeOutput(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.directory = tempfile.TemporaryDirectory()
        cls.addClassCleanup(cls.directory.cleanup)
        cls.root = pathlib.Path(cls.directory.name)
        runtime = (ROOT / "compiler" / "runtime.c").read_text()
        cls.output_bodies = {
            "integer": "P(65);O();",
            "character": "P(44032);Q();",
            "array": "B();P(65);a();P(44032);a();Z();",
        }
        programs = {name: body + "I();" for name, body in cls.output_bodies.items()}
        for instruction in ["I", "K", "T"]:
            programs[f"input-{instruction}"] = f'fputs("pending",stdout);{instruction}();'
        for name, body in cls.output_bodies.items():
            programs[f"unbuffered-{name}"] = "setvbuf(stdout,NULL,_IONBF,0);" + body
        programs["unicode"] = "P(44032);Q();B();P(0x1f980);a();P(0);a();Z();"
        programs["invalid-array"] = "B();P(65);a();P(-1);a();Z();"
        programs["empty-array"] = "B();Z();"
        for name, body in programs.items():
            cls.build(name, runtime + body + "H();return 0;}\n")
        portable = runtime.replace("#include <signal.h>", "#include <signal.h>\n#undef SIGPIPE")
        cls.build("without-sigpipe", portable + "P(65);O();H();return 0;}\n")

    @classmethod
    def build(cls, name, source):
        path = cls.root / f"{name}.c"
        path.write_text(source)
        subprocess.run(
            ["cc", "-std=c11", "-O1", "-o", str(cls.root / name), str(path)],
            check=True, capture_output=True, timeout=30,
        )

    def assert_output_error(self, result):
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(result.stderr, b"haa? haa?\n")

    @unittest.skipUnless(os.name == "posix" and hasattr(signal, "SIGPIPE"), "requires POSIX pipes")
    def test_output_commands_handle_default_and_ignored_sigpipe(self):
        for name, restore in itertools.product(self.output_bodies, [True, False]):
            result = run_with_closed_stdout(self.root / name, restore)
            self.assert_output_error(result)

    @unittest.skipUnless(os.name == "posix" and hasattr(signal, "SIGPIPE"), "requires POSIX pipes")
    def test_output_failure_does_not_consume_following_input(self):
        for name in self.output_bodies:
            result, remaining = run_with_pending_input(self.root / name)
            self.assert_output_error(result)
            self.assertEqual(remaining, b"7\n", name)

    @unittest.skipUnless(os.name == "posix" and hasattr(signal, "SIGPIPE"), "requires POSIX pipes")
    def test_pending_output_is_checked_before_each_input_command(self):
        for instruction in ["I", "K", "T"]:
            result, remaining = run_with_pending_input(self.root / f"input-{instruction}")
            self.assert_output_error(result)
            self.assertEqual(remaining, b"7\n", instruction)

    @unittest.skipUnless(os.name == "posix" and hasattr(signal, "SIGPIPE"), "requires POSIX pipes")
    def test_unbuffered_write_failures_are_handled(self):
        for name in self.output_bodies:
            self.assert_output_error(run_with_closed_stdout(self.root / f"unbuffered-{name}"))

    def test_unicode_and_nul_output_is_unchanged(self):
        result = subprocess.run([str(self.root / "unicode")], capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "가🦀\0".encode())
        self.assertEqual(result.stderr, b"")

    def test_invalid_array_is_validated_before_any_output(self):
        result = subprocess.run([str(self.root / "invalid-array")], capture_output=True, timeout=5)
        self.assert_output_error(result)
        self.assertEqual(result.stdout, b"")

    def test_empty_array_output_is_unchanged(self):
        result = subprocess.run([str(self.root / "empty-array")], capture_output=True, timeout=5)
        self.assertEqual((result.returncode, result.stdout, result.stderr), (0, b"", b""))

    def test_runtime_builds_without_sigpipe(self):
        result = subprocess.run([str(self.root / "without-sigpipe")], capture_output=True, timeout=5)
        self.assertEqual((result.returncode, result.stdout, result.stderr), (0, b"65", b""))


if __name__ == "__main__":
    unittest.main()
