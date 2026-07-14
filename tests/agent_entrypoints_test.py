#!/usr/bin/env python3
"""Behavioral tests for unattended agent entrypoints.

Each case runs copied entrypoints in an isolated git checkout with fake command
binaries. No case calls the user's Foreman, mise, HK, or checkout.
"""

from __future__ import annotations

import json
import os
import shutil
import stat
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class AgentEntrypointTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.temp_path = Path(self.temp.name)
        self.repo = self.temp_path / "fixture-repo"
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
        self.doctor_json.write_text('{"findings": []}\n')
        self.write_fake("mise", """#!/bin/sh
printf 'mise cwd=%s args=%s\\n' "$PWD" "$*" >> "$FAKE_CALL_LOG"
exit "${FAKE_MISE_EXIT:-0}"
""")
        self.write_fake("foreman", """#!/bin/sh
printf 'foreman cwd=%s args=%s\\n' "$PWD" "$*" >> "$FAKE_CALL_LOG"
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

    def entrypoint(self, name: str, *, cwd: Path | None = None, **extra: str) -> subprocess.CompletedProcess[str]:
        state_home = self.temp_path / "state"
        env = {
            "PATH": f"{self.fake_bin}:/usr/bin:/bin",
            "HOME": str(self.temp_path / "home"),
            "XDG_STATE_HOME": str(state_home),
            "FAKE_CALL_LOG": str(self.calls),
            "FAKE_DOCTOR_JSON": str(self.doctor_json),
            **extra,
        }
        return subprocess.run(
            [str(self.repo / ".agents" / name)],
            cwd=cwd or self.temp_path,
            env=env,
            text=True,
            capture_output=True,
            check=False,
        )

    def receipt(self, result: subprocess.CompletedProcess[str]) -> dict[str, object]:
        self.assertTrue(result.stdout, result.stderr)
        return json.loads(result.stdout)

    def test_setup_succeeds_from_another_directory_and_converges(self) -> None:
        first = self.entrypoint("setup")
        second = self.entrypoint("setup", cwd=Path("/"))

        self.assertEqual(first.returncode, 0, first.stderr)
        self.assertEqual(second.returncode, 0, second.stderr)
        first_receipt = self.receipt(first)
        second_receipt = self.receipt(second)
        self.assertTrue(first_receipt["ready"])
        self.assertEqual(first_receipt["schema"], "foreman.agent.setup-receipt")
        self.assertEqual(first_receipt["version"], 1)
        self.assertEqual(first_receipt["doctor"]["report"], {"findings": []})
        self.assertEqual(first_receipt, second_receipt)
        calls = self.calls.read_text()
        self.assertEqual(calls.count("mise cwd="), 2)
        self.assertEqual(calls.count("foreman cwd="), 2)
        physical_repo = self.repo.resolve()
        self.assertIn(f"mise cwd={physical_repo} args=run setup", calls)
        self.assertIn(f"--doctor --doctor-json --doctor-strict --repo {physical_repo}", calls)

    def test_setup_reports_mise_failure_without_running_doctor(self) -> None:
        result = self.entrypoint("setup", FAKE_MISE_EXIT="23")

        self.assertEqual(result.returncode, 23)
        receipt = self.receipt(result)
        self.assertFalse(receipt["ready"])
        self.assertEqual(receipt["setup"]["status"], "failed")
        self.assertEqual(receipt["doctor"]["status"], "not-run")
        self.assertEqual(self.calls.read_text().count("foreman cwd="), 0)
        self.assertIn("mise run setup failed", result.stderr)

    def test_setup_reports_blocking_strict_doctor(self) -> None:
        self.doctor_json.write_text('{"findings": [{"id": "hook-broken", "severity": "error"}]}\n')
        result = self.entrypoint("setup", FAKE_DOCTOR_EXIT="17")

        self.assertEqual(result.returncode, 17)
        receipt = self.receipt(result)
        self.assertFalse(receipt["ready"])
        self.assertEqual(receipt["doctor"]["status"], "blocked")
        self.assertIn({"id": "hook-broken", "severity": "error"}, receipt["errors"])
        self.assertIn({"stage": "doctor", "message": "foreman strict doctor did not pass"}, receipt["errors"])
        self.assertIn("strict doctor blocked readiness", result.stderr)

    def test_resume_reports_clean_dirty_and_detached_without_mutation(self) -> None:
        clean = self.entrypoint("resume")
        clean_report = self.receipt(clean)
        self.assertEqual(clean.returncode, 0, clean.stderr)
        self.assertEqual(clean_report["repo"]["branch"], self.run_git("branch", "--show-current").stdout.strip())
        self.assertFalse(clean_report["repo"]["detached"])
        self.assertFalse(clean_report["repo"]["dirty"])

        (self.repo / "dirty.txt").write_text("uncommitted\n")
        before = self.run_git("status", "--porcelain").stdout
        dirty = self.entrypoint("resume")
        after = self.run_git("status", "--porcelain").stdout
        dirty_report = self.receipt(dirty)
        self.assertEqual(before, after)
        self.assertTrue(dirty_report["repo"]["dirty"])

        (self.repo / "dirty.txt").unlink()
        self.run_git("checkout", "--detach", "-q")
        detached = self.entrypoint("resume")
        detached_report = self.receipt(detached)
        self.assertTrue(detached_report["repo"]["detached"])
        self.assertIsNone(detached_report["repo"]["branch"])
        self.assertIn("foreman cwd=", self.calls.read_text())

    def test_resume_reports_missing_optional_hk_and_logs(self) -> None:
        result = self.entrypoint("resume")

        self.assertEqual(result.returncode, 0, result.stderr)
        report = self.receipt(result)
        self.assertFalse(report["optional_tools"]["hk"]["available"])
        self.assertIsNone(report["logs"]["latest"])
        self.assertTrue(any("hk is not installed" in warning for warning in report["warnings"]))
        self.assertIn("latest Foreman log is not available", report["warnings"])

    def test_resume_reports_an_existing_latest_log_without_reading_it(self) -> None:
        latest = self.temp_path / "state" / "foreman" / "logs" / "latest.log"
        latest.parent.mkdir(parents=True)
        latest.write_text("existing diagnostic\n")

        report = self.receipt(self.entrypoint("resume"))

        self.assertEqual(report["logs"]["latest"], str(latest))
        self.assertEqual(latest.read_text(), "existing diagnostic\n")


if __name__ == "__main__":
    unittest.main(verbosity=2)
