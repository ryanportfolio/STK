"""Real Claude Code + STK hook, using a local scripted Messages endpoint.

python scripts/test-claude-runtime.py --stk target/debug/stk.exe
All model requests go to localhost with a dummy key. No paid API calls.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--stk", required=True)
    parser.add_argument("--claude", default=shutil.which("claude"))
    parser.add_argument("--evidence", type=Path)
    args = parser.parse_args()
    if not args.claude:
        parser.error("claude executable not found")
    stk = Path(args.stk).resolve()
    requests = []
    with tempfile.TemporaryDirectory(prefix="stk-claude-runtime-") as temp:
        root = Path(temp)
        binary = root / "binary with spaces" / stk.name
        binary.parent.mkdir()
        shutil.copyfile(stk, binary)
        binary.chmod(0o755)
        fixture = root / "large file.rs"
        fixture.write_text("".join(f"fn item_{i}() {{}}\n" for i in range(1, 3001)), encoding="utf-8")
        env = {key: value for key, value in os.environ.items() if not key.startswith(("CLAUDE", "ANTHROPIC", "CODEX_"))}
        env.update(CLAUDE_CONFIG_DIR=str(root / ".claude"), STK_DATA_DIR=str(root / "data"),
                   STK_CONFIG_FILE=str(root / "config.toml"), ANTHROPIC_API_KEY="stk-local-fixture-only",
                   CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC="1", DISABLE_TELEMETRY="1", DISABLE_AUTOUPDATER="1")
        subprocess.run([str(binary), "init", "--claude", "--home", temp], env=env, capture_output=True, check=True)

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *unused):
                pass

            def do_POST(self):
                body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
                if "count_tokens" in self.path:
                    self.send_response(200)
                    self.send_header("Content-Type", "application/json")
                    self.end_headers()
                    self.wfile.write(b'{"input_tokens":1}')
                    return
                requests.append(body)
                index = len(requests)
                print(f"Fixture request {index}", flush=True)
                if index <= 2:
                    tool_input = {"file_path": str(fixture)}
                    if index == 2:
                        tool_input.update(offset=2100, limit=2)
                    block = {"type": "tool_use", "id": f"toolu_{index}", "name": "Read", "input": tool_input}
                    stop = "tool_use"
                else:
                    block = {"type": "text", "text": "STK Claude runtime fixture complete."}
                    stop = "end_turn"
                message = {"id": f"msg_{index}", "type": "message", "role": "assistant", "model": body.get("model", "fixture"),
                           "content": [block], "stop_reason": stop, "stop_sequence": None,
                           "usage": {"input_tokens": 1, "output_tokens": 1}}
                if body.get("stream"):
                    start_block = {**block, "input": {}} if index <= 2 else {"type": "text", "text": ""}
                    delta = {"type": "input_json_delta", "partial_json": json.dumps(block["input"])} if index <= 2 else {"type": "text_delta", "text": block["text"]}
                    events = [
                        {"type": "message_start", "message": {**message, "content": [], "stop_reason": None}},
                        {"type": "content_block_start", "index": 0, "content_block": start_block},
                        {"type": "content_block_delta", "index": 0, "delta": delta},
                        {"type": "content_block_stop", "index": 0},
                        {"type": "message_delta", "delta": {"stop_reason": stop, "stop_sequence": None}, "usage": {"output_tokens": 1}},
                        {"type": "message_stop"},
                    ]
                    data = "".join(f"event: {event['type']}\ndata: {json.dumps(event)}\n\n" for event in events).encode()
                    content_type = "text/event-stream"
                else:
                    data = json.dumps(message).encode()
                    content_type = "application/json"
                self.send_response(200)
                self.send_header("Content-Type", content_type)
                self.send_header("Content-Length", str(len(data)))
                self.end_headers()
                self.wfile.write(data)

        server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        env["ANTHROPIC_BASE_URL"] = f"http://127.0.0.1:{server.server_port}"
        try:
            result = subprocess.run([args.claude, "-p", "Run the STK runtime fixture.", "--tools", "Read", "--allowedTools", "Read",
                "--permission-mode", "dontAsk", "--setting-sources", "", "--settings", str(root / ".claude/settings.json"),
                "--strict-mcp-config", "--mcp-config", '{"mcpServers":{}}', "--no-session-persistence",
                "--output-format", "stream-json", "--verbose", "--include-hook-events", "--system-prompt", "Run this local tool fixture."],
                cwd=temp, env=env, stdin=subprocess.DEVNULL, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=45)
        finally:
            server.shutdown()
            server.server_close()
        outputs = {block["tool_use_id"]: block.get("content") for request in requests for message in request.get("messages", [])
                   if isinstance(message.get("content"), list) for block in message["content"] if block.get("type") == "tool_result"}
        if args.evidence:
            args.evidence.write_text(json.dumps({"claude_exit": result.returncode, "request_count": len(requests), "tool_outputs": outputs,
                "stdout": result.stdout, "stderr": result.stderr}, indent=2), encoding="utf-8")
        assert result.returncode == 0, result.stderr + result.stdout
        first = json.dumps(outputs.get("toolu_1"))
        second = json.dumps(outputs.get("toolu_2"))
        assert "stk clamp:" in first and "fn item_2999" not in first, first
        assert "fn item_2100() {}" in second and "fn item_2101() {}" in second and "fn item_2102() {}" not in second, second
        print("PASS: actual Claude loaded the installed hook, suppressed the whole read, and returned the requested range.")


if __name__ == "__main__":
    main()
