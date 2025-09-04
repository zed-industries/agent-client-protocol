import asyncio
import json
import os
import signal
import sys
from pathlib import Path


async def _relay(reader: asyncio.StreamReader, writer: asyncio.StreamWriter, tag: str):
    try:
        while True:
            line = await reader.readline()
            if not line:
                break
            # Mirror to the other end unchanged
            writer.write(line)
            try:
                await writer.drain()
            except ConnectionError:
                break
            # Try to pretty-print the JSON-RPC message for visibility
            try:
                obj = json.loads(line.decode("utf-8", errors="replace"))
                pretty = json.dumps(obj, ensure_ascii=False, indent=2)
                print(f"[{tag}] {pretty}", file=sys.stderr)
            except Exception:
                # Non-JSON (shouldn't happen on the protocol stream)
                print(f"[{tag}] {line!r}", file=sys.stderr)
    finally:
        try:
            writer.close()
            await writer.wait_closed()
        except Exception:
            pass


async def main() -> None:
    root = Path(__file__).resolve().parent
    agent_path = str(root / "agent.py")
    client_path = str(root / "client.py")

    # Ensure PYTHONPATH includes project src for `from acp import ...`
    env = os.environ.copy()
    src_dir = str((root.parent / "src").resolve())
    env["PYTHONPATH"] = src_dir + os.pathsep + env.get("PYTHONPATH", "")

    agent = await asyncio.create_subprocess_exec(
        sys.executable,
        agent_path,
        stdin=asyncio.subprocess.PIPE,
        stdout=asyncio.subprocess.PIPE,
        stderr=sys.stderr,
        env=env,
    )
    client = await asyncio.create_subprocess_exec(
        sys.executable,
        client_path,
        stdin=asyncio.subprocess.PIPE,
        stdout=asyncio.subprocess.PIPE,
        stderr=sys.stderr,
        env=env,
    )

    assert agent.stdout and agent.stdin and client.stdout and client.stdin

    # Wire: agent.stdout -> client.stdin, client.stdout -> agent.stdin
    t1 = asyncio.create_task(_relay(agent.stdout, client.stdin, "agent→client"))
    t2 = asyncio.create_task(_relay(client.stdout, agent.stdin, "client→agent"))

    # Handle shutdown
    stop = asyncio.Event()

    def _on_sigint(*_):
        stop.set()

    loop = asyncio.get_running_loop()
    try:
        loop.add_signal_handler(signal.SIGINT, _on_sigint)
        loop.add_signal_handler(signal.SIGTERM, _on_sigint)
    except NotImplementedError:
        pass

    done, _ = await asyncio.wait(
        {t1, t2, asyncio.create_task(agent.wait()), asyncio.create_task(client.wait()), asyncio.create_task(stop.wait())},
        return_when=asyncio.FIRST_COMPLETED,
    )

    # Teardown
    for proc in (agent, client):
        if proc.returncode is None:
            with contextlib.suppress(ProcessLookupError):
                proc.terminate()
            try:
                await asyncio.wait_for(proc.wait(), 2)
            except asyncio.TimeoutError:
                with contextlib.suppress(ProcessLookupError):
                    proc.kill()
    for task in (t1, t2):
        task.cancel()
        with contextlib.suppress(asyncio.CancelledError):
            await task


if __name__ == "__main__":
    import contextlib

    asyncio.run(main())
