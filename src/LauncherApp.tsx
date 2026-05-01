import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ServerSetup } from "./auth/ServerSetup";
import { LoginScreen } from "./auth/LoginScreen";
import { authStorage } from "./auth/auth-storage";

interface Account {
  id: string;
  label: string;
  server_url: string;
  last_logged_in_at: number;
}

type Step = "list" | "setup" | "login";

interface Pending {
  serverUrl: string;
  label: string;
}

export function LauncherApp() {
  const [accounts, setAccounts] = useState<Account[] | null>(null);
  const [step, setStep] = useState<Step>("list");
  const [pending, setPending] = useState<Pending | null>(null);

  useEffect(() => {
    invoke<Account[]>("list_accounts").then((accs) => {
      setAccounts(accs);
      if (accs.length === 0) setStep("setup");
    });
  }, []);

  async function openAccount(account: Account) {
    await invoke("open_account_window", { accountId: account.id });
  }

  function handleConnected(serverUrl: string, label: string) {
    setPending({ serverUrl, label });
    setStep("login");
  }

  async function handleLoggedIn(token: string) {
    if (!pending) return;
    const acc = await invoke<Account>("add_account", {
      label: pending.label,
      serverUrl: pending.serverUrl,
    });
    await authStorage.setToken(acc.id, token);
    await invoke("open_account_window", { accountId: acc.id });
    // refresh account list
    const accs = await invoke<Account[]>("list_accounts");
    setAccounts(accs);
    setStep("list");
    setPending(null);
  }

  if (step === "setup") {
    return <ServerSetup onConnected={handleConnected} />;
  }

  if (step === "login" && pending) {
    return (
      <LoginScreen
        serverUrl={pending.serverUrl}
        onLoggedIn={handleLoggedIn}
        onBack={() => setStep("setup")}
      />
    );
  }

  return (
    <div style={{ padding: "2rem", maxWidth: 480 }}>
      <h1 style={{ fontSize: "1.75rem", fontWeight: 700, marginBottom: "1.5rem" }}>
        Kryton
      </h1>

      <h2 style={{ fontSize: "1rem", fontWeight: 600, marginBottom: "0.75rem", color: "#555" }}>
        Accounts
      </h2>

      {accounts === null && (
        <p style={{ color: "#888" }}>Loading…</p>
      )}

      {accounts !== null && accounts.length === 0 && (
        <p style={{ color: "#888", marginBottom: "1rem" }}>No accounts yet.</p>
      )}

      {accounts?.map((a) => {
        let host = a.server_url;
        try { host = new URL(a.server_url).host; } catch { /* keep raw */ }
        return (
          <button
            key={a.id}
            onClick={() => openAccount(a)}
            style={{
              display: "block",
              width: "100%",
              textAlign: "left",
              padding: "0.75rem 1rem",
              border: "1px solid #ddd",
              borderRadius: 6,
              marginBottom: "0.5rem",
              background: "#fff",
              cursor: "pointer",
            }}
          >
            <div style={{ fontWeight: 600 }}>{a.label}</div>
            <div style={{ fontSize: "0.8rem", color: "#888" }}>{host}</div>
          </button>
        );
      })}

      <button
        onClick={() => setStep("setup")}
        style={{
          marginTop: "1rem",
          background: "none",
          border: "none",
          color: "#2563eb",
          cursor: "pointer",
          fontSize: "0.9rem",
          padding: 0,
        }}
      >
        + Add account
      </button>
    </div>
  );
}
