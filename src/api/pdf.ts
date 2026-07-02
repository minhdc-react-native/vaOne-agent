import { invoke } from "@tauri-apps/api/core";

export async function generatePdf(report: any, data: any) {
    return await invoke("render_pdf", {
        report, data
    });
}

export async function printPdf(path: string) {
    return await invoke("print_pdf", {
        path
    })
}