"""Offline integration test: real Codex CLI, real STK hook, scripted local model.

Usage: python scripts/test-codex-runtime.py --stk target/debug/stk.exe
Requires Codex with PreToolUse support. No API credentials or paid requests.
Only the generated, inspected test hook bypasses trust for this invocation.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import threading
import uuid
from contextlib import contextmanager
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


@contextmanager
def fixture_directory():
    parent = Path(tempfile.gettempdir()).resolve()
    root = parent / ("stk-runtime-" + uuid.uuid4().hex)
    # mkdir's default inherited ACL lets a nested Windows sandbox read fixtures;
    # TemporaryDirectory's owner-only Windows ACL does not.
    root.mkdir()
    try:
        yield str(root)
    finally:
        assert root.resolve().parent == parent and root.name.startswith("stk-runtime-")
        shutil.rmtree(root, ignore_errors=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--stk", required=True)
    parser.add_argument("--codex", default=shutil.which("codex"))
    parser.add_argument("--evidence", type=Path)
    parser.add_argument("--installed-home", type=Path, help="Use existing Codex hooks and persisted trust; stats stay isolated")
    args = parser.parse_args()
    if not args.codex:
        parser.error("codex executable not found")
    stk = Path(args.stk).resolve()
    requests = []
    with fixture_directory() as temp:
        root = Path(temp)
        # Verify paths containing spaces through the actual client's hook runner.
        binary_dir = root / "binary with spaces"
        binary_dir.mkdir()
        executable = binary_dir / stk.name
        shutil.copyfile(stk, executable)
        executable.chmod(0o755)
        fixture = root / "large file.rs"
        fixture.write_text("".join(f"fn item_{i}() {{}}\n" for i in range(1, 3001)), encoding="utf-8")
        (root / ".codex").mkdir()
        env = {key: value for key, value in os.environ.items() if not key.startswith("CODEX_")}
        env["CODEX_HOME"] = str(root / ".codex")
        env["STK_DATA_DIR"] = str(root / "data")
        env["STK_CONFIG_FILE"] = str(root / "stk-config.toml")
        if not args.installed_home:
            subprocess.run([str(executable), "init", "--codex", "--home", temp], env=env, check=True, capture_output=True)
        command1 = "Get-Content -LiteralPath 'large file.rs'" if os.name == "nt" else "cat 'large file.rs'"
        exe_path = executable.as_posix().replace("'", "''" if os.name == "nt" else "'\"'\"'")
        command2 = ("& " if os.name == "nt" else "") + f"'{exe_path}' read 'large file.rs' --offset 2100 --limit 2"

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *unused):
                pass

            def do_POST(self):
                body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
                requests.append(body)
                print(f"Fixture request {len(requests)}", flush=True)
                index = len(requests)
                if index <= 2:
                    names = [tool.get("name") for tool in body.get("tools", [])]
                    tool = next((n for n in ["exec_command", "shell_command", "shell"] if n in names), None)
                    if tool is None:
                        raise AssertionError(f"No direct shell tool exposed: {names}")
                    command = command1 if index == 1 else command2
                    arguments = {"cmd" if tool == "exec_command" else "command": command}
                    if tool == "exec_command":
                        arguments.update(workdir=temp, max_output_tokens=2000)
                    item = {"type": "function_call", "id": f"fc_{index}", "call_id": f"call_{index}", "name": tool, "arguments": json.dumps(arguments)}
                else:
                    item = {"type": "message", "id": "msg_done", "role": "assistant", "status": "completed", "content": [{"type": "output_text", "text": "STK runtime fixture complete.", "annotations": []}]}
                response = {"id": f"resp_{index}", "object": "response", "status": "completed", "output": [item], "usage": {"input_tokens": 1, "output_tokens": 1, "total_tokens": 2}}
                events = [
                    {"type": "response.created", "response": {**response, "status": "in_progress", "output": []}},
                    {"type": "response.output_item.added", "output_index": 0, "item": item},
                    {"type": "response.output_item.done", "output_index": 0, "item": item},
                    {"type": "response.completed", "response": response},
                ]
                data = "".join(f"event: {event['type']}\ndata: {json.dumps(event)}\n\n" for event in events).encode()
                self.send_response(200)
                self.send_header("Content-Type", "text/event-stream")
                self.send_header("Content-Length", str(len(data)))
                self.end_headers()
                self.wfile.write(data)

        server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        (root / ".codex/config.toml").write_text(f'''
model_provider = "stk_fixture"
model = "stk-fixture"
check_for_update_on_startup = false
[projects.{json.dumps(root.as_posix())}]
trust_level = "trusted"
[model_providers.stk_fixture]
name = "STK offline runtime fixture"
base_url = "http://127.0.0.1:{server.server_port}/v1"
wire_api = "responses"
requires_openai_auth = false
request_max_retries = 0
stream_max_retries = 0
[features]
hooks = true
apps = false
multi_agent = false
enable_request_compression = false
''', encoding="utf-8")
        runtime_options = ["--dangerously-bypass-hook-trust"]
        if args.installed_home:
            env["CODEX_HOME"] = str(args.installed_home.resolve())
            # Override the provider explicitly while retaining the real hook source
            # and its persisted trust. Never write the installed configuration.
            runtime_options = []
            for key, value in {
                "model_provider": "stk_fixture", "model": "stk-fixture", "notify": [],
                "model_providers.stk_fixture.name": "STK local fixture",
                "model_providers.stk_fixture.base_url": f"http://127.0.0.1:{server.server_port}/v1",
                "model_providers.stk_fixture.wire_api": "responses",
                "model_providers.stk_fixture.requires_openai_auth": False,
                "model_providers.stk_fixture.request_max_retries": 0,
                "model_providers.stk_fixture.stream_max_retries": 0,
                "features.hooks": True, "features.apps": False,
                "features.multi_agent": False, "features.enable_request_compression": False,
            }.items():
                runtime_options.extend(["-c", key + "=" + json.dumps(value)])
        try:
            result = subprocess.run([
                args.codex, "exec", "--skip-git-repo-check", "--ephemeral",
                *runtime_options, "--sandbox", "read-only",
                "-c", 'windows.sandbox="unelevated"',
                "-c", 'approval_policy="never"',
                "-C", temp, "--json", "Run the local STK runtime fixture."
            ], env=env, stdin=subprocess.DEVNULL, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=45)
        except subprocess.TimeoutExpired as error:
            if args.evidence:
                args.evidence.write_text(json.dumps({"timeout": True, "requests": requests, "stdout": str(error.stdout), "stderr": str(error.stderr)}, indent=2), encoding="utf-8")
            raise
        finally:
            server.shutdown()
            server.server_close()
        outputs = {item.get("call_id"): item.get("output") for request in requests for item in request.get("input", []) if item.get("type") == "function_call_output"}
        stats = json.loads(subprocess.check_output([str(executable), "gain", "--json"], env=env))
        evidence = {"stats": stats, "persisted_trust": bool(args.installed_home), "codex_exit": result.returncode, "request_count": len(requests), "tool_outputs": outputs, "stdout": result.stdout, "stderr": result.stderr}
        if args.evidence:
            args.evidence.write_text(json.dumps(evidence, indent=2), encoding="utf-8")
        assert result.returncode == 0, result.stderr + result.stdout
        assert stats["clients"]["codex"]["clamps"] == 1, stats
        first = json.dumps(outputs.get("call_1"))
        second = json.dumps(outputs.get("call_2"))
        assert "stk clamp:" in first, first + result.stderr
        assert "fn item_2999" not in first, "Unclamped full file reached the model"
        assert "fn item_2100() {}" in second and "fn item_2101() {}" in second, second
        assert "fn item_2102() {}" not in second, second
        print("PASS: actual Codex loaded the installed hook, suppressed the whole read, and returned the exact requested range.")


if __name__ == "__main__":
    main()
