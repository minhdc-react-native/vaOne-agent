import { check } from "@tauri-apps/plugin-updater";

export async function checkUpdate() {
    const update = await check();

    if (!update) {
        return null;
    }

    return update;
}