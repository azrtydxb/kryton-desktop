import { invoke } from "@tauri-apps/api/core";

export const authStorage = {
  async getToken(accountId: string): Promise<string | null> {
    return invoke<string | null>("get_auth_token", { accountId });
  },
  async setToken(accountId: string, token: string): Promise<void> {
    return invoke("set_auth_token", { accountId, token });
  },
  async clearToken(accountId: string): Promise<void> {
    return invoke("clear_auth_token", { accountId });
  },
};
