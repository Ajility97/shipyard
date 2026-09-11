import { invoke } from "@tauri-apps/api/core";

export async function ensureNotificationPermission() {
  try {
    await invoke("request_notification_permission");
  } catch {
    // macOS may prompt on first send instead
  }
}

export async function notifyRefreshComplete(count: number, groupName?: string) {
  const repos = count === 1 ? "1 repository" : `${count} repositories`;
  const body = groupName
    ? `Refreshed ${groupName} (${repos}).`
    : `Refresh complete. Updated ${repos}.`;
  try {
    await invoke("notify_user", { title: "Krakdown", body });
  } catch {
    // Notifications are best-effort; refresh still succeeded.
  }
}
