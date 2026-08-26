import { spawn, ChildProcess } from "node:child_process";
import http from "node:http";
import net from "node:net";

const REQUESTED_PORT = Number(process.env.AGENTIK_PDF_PORT || 4317);

function waitForReady(url: string, timeoutMs = 60_000): Promise<void> {
  return new Promise((resolve, reject) => {
    const start = Date.now();
    const tick = () => {
      const req = http.get(url, (res) => {
        res.resume();
        if (res.statusCode && res.statusCode < 500) return resolve();
        retry();
      });
      req.on("error", retry);
      req.setTimeout(2000, () => req.destroy());
    };
    const retry = () => {
      if (Date.now() - start > timeoutMs) {
        return reject(new Error(`server not ready after ${timeoutMs}ms`));
      }
      setTimeout(tick, 400);
    };
    tick();
  });
}

let proc: ChildProcess | null = null;

export async function startServer(cwd: string): Promise<string> {
  // Never reuse an arbitrary process already listening on the default port.
  // A different project can be healthy there and still render the wrong PDF.
  // Reserve a free port for this process, then stop only the child we spawn.
  const port = await findFreePort(REQUESTED_PORT);
  const base = `http://127.0.0.1:${port}`;
  proc = spawn("npx", ["next", "start", "--port", String(port)], {
    cwd,
    env: { ...process.env, NODE_ENV: "production" },
    stdio: ["ignore", "pipe", "pipe"],
    detached: true
  });
  proc.stdout?.on("data", (d) => process.stderr.write(`[next] ${d}`));
  proc.stderr?.on("data", (d) => process.stderr.write(`[next] ${d}`));
  await waitForReady(base, 60_000);
  return base;
}

async function findFreePort(preferred: number): Promise<number> {
  const candidates = preferred > 0 ? [preferred] : [];
  candidates.push(0);
  for (const candidate of candidates) {
    const port = await new Promise<number | null>((resolve) => {
      const probe = net.createServer();
      probe.once("error", () => resolve(null));
      probe.listen(candidate, "127.0.0.1", () => {
        const address = probe.address();
        const selected = typeof address === "object" && address ? address.port : null;
        probe.close(() => resolve(selected));
      });
    });
    if (port !== null) return port;
  }
  throw new Error(`no free PDF renderer port near ${preferred}`);
}

export async function stopServer(): Promise<void> {
  if (proc && !proc.killed) {
    try {
      // Kill the entire process group to catch Next's child processes too
      process.kill(-proc.pid!, "SIGTERM");
    } catch {
      proc.kill("SIGTERM");
    }
    await new Promise((r) => setTimeout(r, 400));
    if (!proc.killed) {
      try {
        process.kill(-proc.pid!, "SIGKILL");
      } catch {
        proc.kill("SIGKILL");
      }
    }
  }
  proc = null;
}
