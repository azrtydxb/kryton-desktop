import { useState } from "react";

interface LoginScreenProps {
  serverUrl: string;
  onLoggedIn: (token: string) => void;
  onBack?: () => void;
}

export function LoginScreen({ serverUrl, onLoggedIn, onBack }: LoginScreenProps) {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  let host = serverUrl;
  try {
    host = new URL(serverUrl).host;
  } catch {
    // fall back to raw serverUrl
  }

  async function login() {
    setError(null);
    setBusy(true);
    try {
      const res = await fetch(`${serverUrl}/api/auth/sign-in/email`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, password }),
      });
      if (!res.ok) throw new Error(`Login failed: ${res.status}`);
      const body = await res.json();
      const token: string | undefined = body.token ?? body.session?.token;
      if (!token) throw new Error("No token returned by server");
      onLoggedIn(token);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Login failed");
    } finally {
      setBusy(false);
    }
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter" && !busy) login();
  }

  return (
    <div style={{ padding: "2rem", maxWidth: 420 }}>
      <h1 style={{ marginBottom: "1rem", fontSize: "1.5rem", fontWeight: 600 }}>
        Log in to {host}
      </h1>

      <label style={{ display: "block", marginBottom: "0.25rem", fontSize: "0.875rem" }}>
        Email
      </label>
      <input
        style={{ display: "block", width: "100%", padding: "0.5rem", marginBottom: "1rem", boxSizing: "border-box", border: "1px solid #ccc", borderRadius: 4 }}
        type="email"
        value={email}
        onChange={(e) => setEmail(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="you@example.com"
        autoFocus
        disabled={busy}
      />

      <label style={{ display: "block", marginBottom: "0.25rem", fontSize: "0.875rem" }}>
        Password
      </label>
      <input
        style={{ display: "block", width: "100%", padding: "0.5rem", marginBottom: "1rem", boxSizing: "border-box", border: "1px solid #ccc", borderRadius: 4 }}
        type="password"
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="Password"
        disabled={busy}
      />

      {error && (
        <p style={{ color: "#c0392b", marginBottom: "1rem", fontSize: "0.875rem" }}>
          {error}
        </p>
      )}

      <div style={{ display: "flex", gap: "0.75rem", alignItems: "center" }}>
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
          onClick={login}
          disabled={busy}
        >
          {busy ? "Logging in…" : "Log in"}
        </button>
        {onBack && (
          <button
            style={{ background: "none", border: "none", color: "#2563eb", cursor: "pointer", fontSize: "0.875rem" }}
            onClick={onBack}
            disabled={busy}
          >
            Back
          </button>
        )}
      </div>
    </div>
  );
}
