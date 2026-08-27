#!/usr/bin/env python3
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CANONICAL_SPEC = [
    "Summary",
    "Goals / Non-Goals",
    "Requirements",
    "Interfaces & Contracts",
    "Invariants",
    "Acceptance",
]


def main() -> None:
    spec_headings = [
        line.removeprefix("## ")
        for line in (ROOT / "SPEC.md").read_text().splitlines()
        if line.startswith("## ")
    ]
    assert spec_headings == CANONICAL_SPEC, spec_headings

    decisions = sorted((ROOT / "docs/decisions").glob("*.md"))
    required_decisions = {
        "0001-stack-choice.md",
        "0002-source-aggregation-and-remote-ssh.md",
        "0003-remote-jump-terminal-activation.md",
        "0004-source-companion-relay.md",
        "0005-native-provenance-authority.md",
    }
    assert required_decisions <= {decision.name for decision in decisions}
    for decision in decisions:
        text = decision.read_text()
        assert re.search(r"^status: (accepted|proposed|deprecated|rejected|superseded)$", text, re.M), decision
        assert re.search(r"^date: \d{4}-\d{2}-\d{2}$", text, re.M), decision
        assert "## Context" in text, decision
        assert "## Decision" in text or "## Decision summary" in text, decision

    adr2 = (ROOT / "docs/decisions/0002-source-aggregation-and-remote-ssh.md").read_text()
    for private_example in (
        "alex.furrier",
        "alex-furrier-dev-gpu-1",
        "coder-dev-gpu-1",
        "/home/discord/.cargo/bin/foreman",
    ):
        assert private_example not in adr2

    plans = (ROOT / ".ai/plans/AGENTS.md").read_text()
    assert "Do not create a new plan" not in plans
    assert "Never add a new plan" in plans
    assert "hk start" in plans

    for skill_name in ("plan-sync", "spec-sync"):
        primary = ROOT / f".agent/skills/{skill_name}/SKILL.md"
        compatibility = ROOT / f".agents/skills/{skill_name}/SKILL.md"
        assert primary.read_bytes() == compatibility.read_bytes()

    spec_sync = (ROOT / ".agent/skills/spec-sync/SKILL.md").read_text()
    assert "local steps below" in spec_sync
    assert "If missing, report that gap" in spec_sync

    docs_index = (ROOT / "docs/README.md").read_text()
    for decision in decisions:
        assert decision.name in docs_index
    assert "project-evolution.md" in docs_index


if __name__ == "__main__":
    main()
