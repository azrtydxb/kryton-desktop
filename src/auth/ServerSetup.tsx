import { useState } from "react";

interface ServerSetupProps {
  onConnected: (serverUrl: string, label: string) => void;
}

export function ServerSetup({ onConnected }: ServerSetupProps) {
  const [url, setUrl] = useState("https://");
  const [label, setLabel] = useState("My Kryton");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function connect() {
    setError(null);
    setBusy(true);
    try {
      const baseUrl = url.replace(/\/$/, "");
      const probe = await fetch(`${baseUrl}/api/version`);
      if (!probe.ok) throw new Error(`Server returned ${probe.status}`);
      const v = await probe.json();
      if (!v.version) throw new Error("Not a Kryton server");
      onConnected(baseUrl, label.trim() || "My Kryton");
    } catch (e) {
      setError(e instanceof Error ? e.message : "Connection failed");
    } finally {
      setBusy(false);
    }
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter" && !busy) connect();
  }

  return (
    <div style={{ padding: "2rem", maxWidth: 420 }}>
      <h1 style={{ marginBottom: "1rem", fontSize: "1.5rem", fontWeight: 600 }}>
        Connect to your Kryton server
      </h1>

      <label style={{ display: "block", marginBottom: "0.25rem", fontSize: "0.875rem" }}>
        Server URL
      </label>
      <input
        style={{ display: "block", width: "100%", padding: "0.5rem", marginBottom: "1rem", boxSizing: "border-box", border: "1px solid #ccc", borderRadius: 4 }}
        value={url}
        onChange={(e) => setUrl(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="https://kryton.example.com"
        autoFocus
        disabled={busy}
      />

      <label style={{ display: "block", marginBottom: "0.25rem", fontSize: "0.875rem" }}>
        Account label
      </label>
      <input
        style={{ display: "block", width: "100%", padding: "0.5rem", marginBottom: "1rem", boxSizing: "border-box", border: "1px solid #ccc", borderRadius: 4 }}
        value={label}
        onChange={(e) => setLabel(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="e.g. Personal"
        disabled={busy}
      />

      {error && (
        <p style={{ color: "#c0392b", marginBottom: "1rem", fontSize: "0.875rem" }}>
          {error}
        </p>
      )}

      <button
        style={{
          padding: "0.5rem 1.25rem",
          background: busy ? "#aaa" : "#2563eb",
          color: "#fff",
          border: "none",
          borderRadius: 4,
          cursor: busy ? "not-allowed" : "pointer",
          fontSize: "1rem",
        }}
        onClick={connect}
        disabled={busy}
      >
        {busy ? "Connecting…" : "Connect"}
      </button>
    </div>
  );
}
