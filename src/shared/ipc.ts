import { invoke } from "@tauri-apps/api/core";

export type Account = {
  id: string;
  label: string;
  server_url: string;
  username: string;
  last_active: string;
};

export const ipc = {
  listAccounts: () => invoke<Account[]>("list_accounts"),
  loginAndAdd: (args: {
    label: string;
    serverUrl: string;
    username: string;
    password: string;
  }) =>
    invoke<Account>("login_and_add", {
      label: args.label,
      serverUrl: args.serverUrl,
      username: args.username,
      password: args.password,
    }),
  silentRelogin: (accountId: string) =>
    invoke<void>("silent_relogin", { accountId }),
  removeAccount: (accountId: string) =>
    invoke<void>("remove_account", { accountId }),
  refreshMenu: () => invoke<void>("refresh_menu"),
};

export const windows = {
  openServer: (accountId: string) => invoke<void>("open_server", { accountId }),
  switchTo: (accountId: string) => invoke<void>("switch_to", { accountId }),
  closeServer: (accountId: string) => invoke<void>("close_server", { accountId }),
};
