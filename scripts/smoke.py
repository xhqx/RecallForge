#!/usr/bin/env python3
"""End-to-end CLI, storage, backup and stdio MCP checks with synthetic data only."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
BIN = Path(os.environ.get("RECALLFORGE_BIN", ROOT / "target/debug/recallforge"))

def main():
    checks = 0
    with tempfile.TemporaryDirectory(prefix="recallforge-smoke-") as temp:
        root = Path(temp)
        data = root / "data"
        def cli(*args, ok=True):
            proc = subprocess.run([str(BIN), "--data-dir", str(data), *args], text=True, capture_output=True, timeout=30)
            if ok:
                assert proc.returncode == 0, proc.stderr
                return json.loads(proc.stdout)
            assert proc.returncode != 0, proc.stdout
            return proc.stderr
        assert cli("init")["ready"]
        cli("project", "add", "--id", "demo", "--name", "Demo", "--root", str(root))
        entry = {"title": "Register dependencies in the test container", "problem": "Integration handler fails to resolve a dependency", "solution": "Register service in test module", "verification": "synthetic smoke fixture", "status": "verified", "sources": [{"reference": "tests/container.rs"}]}
        note = root / "entry.json"
        note.write_text(json.dumps(entry))
        a = cli("save", "--project", "demo", "--file", str(note), "--lexical-only")
        assert a["revision"] == 1 and not a["indexed"]
        assert cli("save", "--project", "demo", "--file", str(note), "--lexical-only")["id"] == a["id"]
        assert cli("search", "--project", "demo", "--query", "dependency", "--mode", "lexical")["hits"][0]["id"] == a["id"]
        entry.update(id=a["id"], expected_revision=1, cause="Separate test application")
        note.write_text(json.dumps(entry))
        assert cli("save", "--project", "demo", "--file", str(note), "--lexical-only")["revision"] == 2
        assert "revision conflict" in cli("save", "--project", "demo", "--file", str(note), "--lexical-only", ok=False)
        assert len(cli("history", "--project", "demo", "--id", a["id"])) == 2
        cli("get", "--project", "absent", "--id", a["id"], ok=False)
        cli("backup", "--output", str(root / "backup.db"))
        cli("backup", "--output", str(root / "backup.db"), ok=False)
        assert cli("doctor")["integrity"] == "ok"
        if os.name == "posix":
            assert data.stat().st_mode & 0o777 == 0o700
            assert (data / "memory.db").stat().st_mode & 0o777 == 0o600
        checks += 12
        messages = [
            {"jsonrpc": "2.0", "id": 0, "method": "tools/list"},
            {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "smoke", "version": "1"}}},
            {"jsonrpc": "2.0", "method": "notifications/initialized"},
            {"jsonrpc": "2.0", "id": 2, "method": "tools/list"},
            {"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "memory_search", "arguments": {"project": "demo", "query": "dependency", "mode": "lexical"}}},
            {"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "memory_get", "arguments": {"project": "absent", "id": a["id"]}}},
            {"jsonrpc": "2.0", "id": 5, "method": "not_a_method"},
            {"jsonrpc": "2.0", "id": 6, "method": "tools/call", "params": {"name": "memory_save", "arguments": {"project": "demo", "entry": {"title": "No evidence", "problem": "unknown", "status": "verified"}, "lexical_only": True}}},
            {"jsonrpc": "2.0", "id": 7, "method": "tools/call", "params": {"name": "memory_get", "arguments": {"project": "demo", "id": a["id"]}}},
        ]
        proc = subprocess.run([str(BIN), "--data-dir", str(data), "mcp"], input="\n".join(json.dumps(m) for m in messages) + "\n{broken\n", text=True, capture_output=True, timeout=30)
        assert proc.returncode == 0, proc.stderr
        replies = [json.loads(line) for line in proc.stdout.splitlines()]
        assert len(replies) == 9
        assert replies[0]["error"]["code"] == -32002
        assert replies[1]["result"]["serverInfo"]["name"] == "recallforge"
        assert len(replies[2]["result"]["tools"]) == 10
        assert not replies[3]["result"]["isError"]
        assert json.loads(replies[3]["result"]["content"][0]["text"])["hits"][0]["id"] == a["id"]
        assert replies[4]["result"]["isError"]
        assert replies[5]["error"]["code"] == -32601
        assert replies[6]["result"]["isError"]
        assert json.loads(replies[7]["result"]["content"][0]["text"])["revision"] == 2
        assert replies[8]["error"]["code"] == -32700
        checks += 11
    print(f"CLI/MCP smoke: {checks} checks passed")

if __name__ == "__main__":
    main()
