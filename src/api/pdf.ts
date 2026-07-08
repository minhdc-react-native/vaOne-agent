import { invoke } from "@tauri-apps/api/core";
import { writeFile } from "@tauri-apps/plugin-fs";
import { tempDir } from "@tauri-apps/api/path";

export async function generatePdf(report: any, data: any) {
    // const dir = await tempDir();
    // const path_report = `${dir}/report.json`;
    // const path_data = `${dir}/report.json`;

    // const encoder = new TextEncoder();

    // await writeFile(
    //     path_report,
    //     encoder.encode(JSON.stringify(report))
    // );

    // await writeFile(
    //     path_data,
    //     encoder.encode(JSON.stringify(data))
    // );

    return await invoke("render_pdf", {
        report, data
    });
}

export async function printPdf(path: string) {
    return await invoke("print_pdf", {
        path
    })
}