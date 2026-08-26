export default function Home() {
  return (
    <main className="p-12 max-w-3xl mx-auto font-sans">
      <h1 className="h-1 mb-4">Agentik PDF</h1>
      <p className="body">
        Unified PDF rendering service. Templates live at{" "}
        <code className="font-mono text-sm">/render/&lt;template&gt;</code>.
      </p>
      <ul className="mt-6 space-y-2 text-sm">
        <li><a className="underline" href="/render/whitepaper?demo=1">whitepaper (demo)</a></li>
        <li><a className="underline" href="/render/audit?demo=1">audit (demo)</a></li>
        <li><a className="underline" href="/render/marketing?demo=1">marketing (demo)</a></li>
        <li><a className="underline" href="/render/doc?demo=1">doc (demo)</a></li>
      </ul>
    </main>
  );
}
