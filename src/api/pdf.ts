import { invoke } from "@tauri-apps/api/core";
import { writeFile } from "@tauri-apps/plugin-fs";
import { tempDir } from "@tauri-apps/api/path";

export async function generatePdf(reports: any, datas: any) {
    return await invoke("render_pdf", {
        reports, datas
    });
}

export async function printPdf(path: string) {
    return await invoke("print_pdf", {
        path
    })
}