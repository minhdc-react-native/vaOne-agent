import { invoke } from "@tauri-apps/api/core";

export async function generatePdf(document: any) {
    return await invoke("render_pdf", {
        document
    });
}

export async function printPdf(path: string) {
    return await invoke("print_pdf", {
        path
    })
}