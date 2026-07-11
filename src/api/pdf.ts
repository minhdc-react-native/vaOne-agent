import { invoke } from "@tauri-apps/api/core";
import { writeFile } from "@tauri-apps/plugin-fs";
import { tempDir } from "@tauri-apps/api/path";

export async function generatePdf(reports: any, datas: any) {
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
        reports, datas
    });
}

export async function printPdf(path: string) {
    return await invoke("print_pdf", {
        path
    })
}