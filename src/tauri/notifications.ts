import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

let permissionChecked = false;
let permitted = false;

async function ensurePermission(): Promise<boolean> {
  if (permissionChecked) return permitted;
  permissionChecked = true;
  permitted = await isPermissionGranted();
  if (!permitted) permitted = (await requestPermission()) === "granted";
  return permitted;
}

export async function notify(title: string, body?: string): Promise<void> {
  if (!(await ensurePermission())) return;
  sendNotification({ title, body });
}
