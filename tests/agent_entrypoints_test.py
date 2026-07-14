#!/usr/bin/env python3
"""Behavioral tests for unattended agent entrypoints using isolated command fakes."""

from __future__ import annotations

import json
import shutil
import stat
import subprocess
import tempfile
import unittest
from pathlib import Path
from typing import cast

ROOT = Path(__file__).resolve().parents[1]
EMPTY_DOCTOR = {"repo_path": None, "findings": [], "fixes": []}
FINDING = {
    "id": "hook-broken", "severity": "error", "area": "repo", "provider": None,
    "pane_id": None, "repo_path": None, "summary": "Hook is broken", "detail": None,
    "evidence": [], "next_step": None,
}


class AgentEntrypointTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.temp_path = Path(self.temp.name)
        self.repo = self.temp_path / 'fixture repo "quoted"'
        (self.repo / ".agents").mkdir(parents=True)
        for name in ("setup", "resume", "entrypoint-report.py"):
            target = self.repo / ".agents" / name
            shutil.copy2(ROOT / ".agents" / name, target)
            target.chmod(target.stat().st_mode | stat.S_IXUSR)
        (self.repo / "README.md").write_text("fixture\n")
        self.run_git("init", "-q")
        self.run_git("add", "README.md", ".agents")
        self.run_git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.test", "commit", "-qm", "fixture")
        self.fake_bin = self.temp_path / "fake-bin"
        self.fake_bin.mkdir()
        self.calls = self.temp_path / "calls.log"
        self.doctor_json = self.temp_path / "doctor.json"
        self.doctor_json.write_text(json.dumps(EMPTY_DOCTOR))
        self.mise_state = self.temp_path / "mise-state"
        self.write_fake("mise", """#!/bin/sh
printf 'mise cwd=%s args=%s\\n' "$PWD" "$*" >> "$FAKE_CALL_LOG"
if [ "$1 $2" = "run setup" ]; then
  if [ ! -f "$FAKE_MISE_STATE" ]; then
    printf 'configured\\n' > "$FAKE_MISE_STATE.tmp"
    mv "$FAKE_MISE_STATE.tmp" "$FAKE_MISE_STATE"
  fi
fi
exit "${FAKE_MISE_EXIT:-0}"
""")
        self.write_fake("foreman", """#!/bin/sh
printf 'foreman cwd=%s args=%s\\n' "$PWD" "$*" >> "$FAKE_CALL_LOG"
printf '%s\\n' "${FAKE_FOREMAN_STDERR:-}" >&2
cat "$FAKE_DOCTOR_JSON"
exit "${FAKE_DOCTOR_EXIT:-0}"
""")

    def tearDown(self) -> None:
        self.temp.cleanup()

    def write_fake(self, name: str, content: str) -> None:
        path = self.fake_bin / name
        path.write_text(content)
        path.chmod(0o755)

    def run_git(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(["git", *args], cwd=self.repo, check=True, text=True, capture_output=True)

    def entrypoint(self, name: str, *, cwd: Path | None = None, hk: bool = False, **extra: str) -> subprocess.CompletedProcess[str]:
        if hk:
            self.write_fake("hk", """#!/bin/sh
printf 'hk stderr from fake\\n' >&2
printf '%s\\n' "$FAKE_HK_JSON"
exit "${FAKE_HK_EXIT:-0}"
""")
        env = {
            "PATH": f"{self.fake_bin}:/usr/bin:/bin", "HOME": str(self.temp_path / "home"),
            "XDG_STATE_HOME": str(self.temp_path / "state"), "FAKE_CALL_LOG": str(self.calls),
            "FAKE_DOCTOR_JSON": str(self.doctor_json), "FAKE_MISE_STATE": str(self.mise_state), **extra,
        }
        return subprocess.run([str(self.repo / ".agents" / name)], cwd=cwd or self.temp_path, env=env, text=True, capture_output=True, check=False)

    def receipt(self, result: subprocess.CompletedProcess[str]) -> dict[str, object]:
        self.assertTrue(result.stdout, result.stderr)
        return cast(dict[str, object], json.loads(result.stdout))

    def messages(self, receipt: dict[str, object], key: str) -> list[dict[str, object]]:
        return cast(list[dict[str, object]], receipt[key])

    def assert_command(self, value: object, argv: list[str]) -> None:
        command = cast(dict[str, object], value)
        self.assertEqual(command["argv"], argv)
        self.assertEqual(command["cwd"], str(self.repo.resolve()))

    def assert_message_schema(self, messages: list[dict[str, object]]) -> None:
        for item in messages:
            self.assertEqual(set(item), {"status", "stage", "code", "message", "next_command", "finding"})
            self.assertIn(item["status"], {"warning", "error"})

    def test_setup_portable_convergent_and_structured_commands(self) -> None:
        before = self.run_git("status", "--porcelain").stdout
        first = self.entrypoint("setup", cwd=Path("/"))
        state_after_first = self.mise_state.read_text()
        second = self.entrypoint("setup", cwd=Path("/"))
        after = self.run_git("status", "--porcelain").stdout
        first_receipt = self.receipt(first)
        second_receipt = self.receipt(second)
        self.assertEqual(first.returncode, 0, first.stderr)
        self.assertEqual(second.returncode, 0, second.stderr)
        self.assertEqual(before, after)
        self.assertEqual(state_after_first, self.mise_state.read_text())
        self.assertEqual(first_receipt, second_receipt)
        commands = cast(dict[str, object], first_receipt["commands"])
        self.assert_command(commands["setup"], [str(self.repo.resolve() / ".agents" / "setup")])
        self.assert_command(commands["doctor"], ["foreman", "--doctor", "--doctor-json", "--repo", str(self.repo.resolve())])
        self.assert_command(commands["focused"], ["python3", "tests/agent_entrypoints_test.py"])
        self.assertIn(f"foreman cwd={self.repo.resolve()} args=--doctor --doctor-json --doctor-strict --repo {self.repo.resolve()}", self.calls.read_text())

    def test_setup_failure_receipt_has_actionable_message(self) -> None:
        result = self.entrypoint("setup", FAKE_MISE_EXIT="23")
        receipt = self.receipt(result)
        self.assertEqual(result.returncode, 23)
        error = self.messages(receipt, "errors")[0]
        self.assertEqual(error["code"], "setup-failed")
        self.assert_message_schema(self.messages(receipt, "errors"))
        self.assert_command(error["next_command"], [str(self.repo.resolve() / ".agents" / "setup")])
        self.assertIn("mise run setup failed", result.stderr)

    def test_missing_mise_and_foreman_have_actionable_stderr_and_receipts(self) -> None:
        no_mise = self.entrypoint("setup", PATH="/usr/bin:/bin")
        no_mise_receipt = self.receipt(no_mise)
        self.assertEqual(no_mise.returncode, 127)
        self.assertIn(str(self.repo.resolve() / ".agents" / "setup"), no_mise.stderr)
        self.assert_command(self.messages(no_mise_receipt, "errors")[0]["next_command"], [str(self.repo.resolve() / ".agents" / "setup")])
        self.write_fake("mise", "#!/bin/sh\nexit 0\n")
        (self.fake_bin / "foreman").unlink()
        no_foreman = self.entrypoint("setup")
        no_foreman_receipt = self.receipt(no_foreman)
        self.assertEqual(no_foreman.returncode, 127)
        self.assertIn("mise run install-local", no_foreman.stderr)
        self.assert_command(cast(dict[str, object], no_foreman_receipt["commands"])["install_local"], ["mise", "run", "install-local"])
        self.assert_command(self.messages(no_foreman_receipt, "errors")[0]["next_command"], ["mise", "run", "install-local"])

    def test_doctor_failure_and_malformed_valid_json_are_visible_errors(self) -> None:
        self.doctor_json.write_text(json.dumps({**EMPTY_DOCTOR, "findings": [FINDING]}))
        blocked = self.entrypoint("setup", FAKE_DOCTOR_EXIT="17")
        blocked_receipt = self.receipt(blocked)
        self.assertEqual(blocked.returncode, 17)
        self.assertTrue(any(item["code"] == "doctor-finding" for item in self.messages(blocked_receipt, "errors")))
        for malformed in ([], {}, {"findings": {}}):
            self.doctor_json.write_text(json.dumps(malformed))
            result = self.entrypoint("resume")
            receipt = self.receipt(result)
            self.assertEqual(result.returncode, 0)
            self.assertIsNone(cast(dict[str, object], receipt["doctor"])["report"])
            self.assertTrue(any(item["code"] == "doctor-schema-invalid" for item in self.messages(receipt, "errors")))
            self.assert_message_schema(self.messages(receipt, "errors"))

    def test_resume_read_only_reports_git_states_and_doctor_failure(self) -> None:
        clean = self.receipt(self.entrypoint("resume"))
        self.assertFalse(cast(dict[str, object], clean["repo"])["dirty"])
        (self.repo / "dirty.txt").write_text("uncommitted\n")
        before = self.run_git("status", "--porcelain").stdout
        dirty = self.receipt(self.entrypoint("resume"))
        self.assertEqual(before, self.run_git("status", "--porcelain").stdout)
        self.assertTrue(cast(dict[str, object], dirty["repo"])["dirty"])
        (self.repo / "dirty.txt").unlink()
        self.run_git("checkout", "--detach", "-q")
        detached = self.receipt(self.entrypoint("resume", FAKE_DOCTOR_EXIT="9"))
        self.assertTrue(cast(dict[str, object], detached["repo"])["detached"])
        self.assertTrue(any(item["code"] == "doctor-unavailable" for item in self.messages(detached, "errors")))

    def test_log_resolution_matches_repo_relative_absolute_xdg_and_home_contract(self) -> None:
        cases = [
            ("FOREMAN_LOG_DIR", "relative state", self.repo / "relative state"),
            ("FOREMAN_LOG_DIR", str(self.temp_path / "absolute state"), self.temp_path / "absolute state"),
            ("XDG_STATE_HOME", "relative xdg", self.repo / "relative xdg" / "foreman" / "logs"),
        ]
        for variable, value, directory in cases:
            latest = directory / "latest.log"
            latest.parent.mkdir(parents=True, exist_ok=True)
            latest.write_text("existing diagnostic\n")
            report = self.receipt(self.entrypoint("resume", cwd=Path("/"), **{variable: value}))
            logs = cast(dict[str, object], report["logs"])
            expected_directory = directory if Path(value).is_absolute() else directory.resolve()
            expected_latest = latest if Path(value).is_absolute() else latest.resolve()
            self.assertEqual(logs["directory"], str(expected_directory))
            self.assertEqual(logs["latest"], str(expected_latest))
            self.assertEqual(latest.read_text(), "existing diagnostic\n")
        home = self.temp_path / "custom home"
        latest = home / ".local" / "state" / "foreman" / "logs" / "latest.log"
        latest.parent.mkdir(parents=True)
        latest.touch()
        report = self.receipt(self.entrypoint("resume", cwd=Path("/"), HOME=str(home), XDG_STATE_HOME=""))
        self.assertEqual(cast(dict[str, object], report["logs"])["latest"], str(latest))

    def test_hk_success_and_failure_preserve_stderr(self) -> None:
        successful = self.entrypoint("resume", hk=True, FAKE_HK_JSON='{"active_work":"fixture"}')
        successful_receipt = self.receipt(successful)
        self.assertEqual(successful.returncode, 0, successful.stderr)
        self.assertEqual(cast(dict[str, object], cast(dict[str, object], successful_receipt["optional_tools"])["hk"])["status"], {"active_work": "fixture"})
        self.assertIn("hk stderr from fake", successful.stderr)
        failed = self.entrypoint("resume", hk=True, FAKE_HK_EXIT="4")
        failed_receipt = self.receipt(failed)
        self.assertEqual(failed.returncode, 0, failed.stderr)
        self.assertIn("hk stderr from fake", failed.stderr)
        self.assertIn("hk status was unavailable", failed.stderr)
        self.assertTrue(any(item["code"] == "hk-status-unavailable" for item in self.messages(failed_receipt, "warnings")))
        self.assert_message_schema(self.messages(failed_receipt, "warnings"))


if __name__ == "__main__":
    unittest.main(verbosity=2)
