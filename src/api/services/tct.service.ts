import axios from "axios";
import { getDelayRequest, getTokenTct } from "../../stores/app.store";
import { invoke } from "@tauri-apps/api/core";
import { dialog } from "../../service/dialog.service";
export function formatDate(dateStr?: string): string {
    if (!dateStr) return "";
    const [year, month, day] = dateStr.split("T")[0].split("-");
    return `${day}/${month}/${year}`;
}
interface IInfoLogin {
    username: string;
    password: string;
    ckey: string;
    cvalue: string;
}
interface ILoginRequest {
    token: string;
}
interface IFilterInvoice {
    fromDate: string;
    toDate: string;
}
interface IDataPage {
    datas: any[];
    state: string;
    time: number;
    total: number;
}
export const tctService = {
    async getCaptcha() {
        try {
            const res = await axios.get(
                "https://hoadondientu.gdt.gov.vn/api/captcha"
            );
            return {
                captcha: `data:image/svg+xml;charset=utf-8,${encodeURIComponent(res.data.content)}`,
                key: res.data.key ?? ""
            }
        } catch (e) {
            await dialog.error("Không thể lấy mã Captcha!");
            // await message("Không thể lấy mã Captcha!", {
            //     title: "vaOne plugin",
            //     kind: "error", // "info" | "warning" | "error"
            // });
            return null;
        }
    },
    async login(data: IInfoLogin) {
        try {
            const res = await axios.post(
                "https://hoadondientu.gdt.gov.vn/api/security-taxpayer/authenticate",
                data
            );
            return res.data as ILoginRequest;
        } catch (e) {
            let errorMessage = "Đã xảy ra lỗi.";
            if (axios.isAxiosError(e)) {
                errorMessage =
                    e.response?.data?.message ??
                    e.message;
            }
            await dialog.error(errorMessage);
            return null;
        }
    },
    async getInvoice(type: 1 | 2 | 3 | 4, filter: IFilterInvoice) {
        /*
            1: Mua vào
            2: Bán ra
            3: Mua vào MTT
            4: Bán ra MTT
        */
        const sizePage = 50;
        let url = `https://hoadondientu.gdt.gov.vn/api/query/invoices/sold?sort=tdlap:desc&size=${sizePage}&search=tdlap=ge=${formatDate(filter.fromDate)}T00:00:00;tdlap=le=${formatDate(filter.toDate)}T23:59:59`;
        switch (type) {
            case 1:
            case 3:
                url = `https://hoadondientu.gdt.gov.vn/api/query/invoices/purchase?sort=tdlap:desc&size=${sizePage}&search=tdlap=ge=${formatDate(filter.fromDate)}T00:00:00;tdlap=le=${formatDate(filter.toDate)}T23:59:59;ttxly==5`;
                break;
            case 2:
            case 4:
                url = `https://hoadondientu.gdt.gov.vn/api/query/invoices/sold?sort=tdlap:desc&size=${sizePage}&search=tdlap=ge=${formatDate(filter.fromDate)}T00:00:00;tdlap=le=${formatDate(filter.toDate)}T23:59:59`;
                break;
        }
        return await fetchAll(url);
    },
    async getInvoiceDetail(datas: any[], callBack: (numInvoice: number, currentInvoice: any, isError?: boolean) => void) {
        const token = getTokenTct();
        const delay = getDelayRequest();
        const result: any[] = [];
        for (let i = 0; i < datas.length; i++) {
            try {
                callBack(i + 1, datas[i]);

                const { nbmst, khhdon, shdon, khmshdon } = datas[i];

                const url = `https://hoadondientu.gdt.gov.vn/api/query/invoices/detail?nbmst=${nbmst}&khhdon=${khhdon}&shdon=${shdon}&khmshdon=${khmshdon}`;

                console.log("call:", i, url);

                const res = await invoke("get_invoice_detail", {
                    url,
                    token,
                    delay
                });

                console.log("response:", i, res);

                result.push(...(Array.isArray(res) ? res : [res]));

            } catch (err) {
                console.error("FAILED INDEX:", i, err);
                callBack(i + 1, datas[i], true);
                continue; // 👈 quan trọng: không dừng loop
            }
        }
        return result;
    }
}
export const sleep = (ms?: number) => new Promise(resolve => {
    const fixMs = ms || getDelayRequest();
    setTimeout(resolve, fixMs)
});

async function fetchAll(url: string) {
    const token = getTokenTct();
    const result: any[] = [];
    let hasMore = true;
    let fixUrl = url;
    while (hasMore) {
        console.log('fixUrl>>', fixUrl);
        const res: IDataPage = (await axios.get(fixUrl, {
            headers: {
                Authorization: `Bearer ${token}`,
            },
        })).data;
        result.push(...res.datas);
        hasMore = res.state !== null;
        fixUrl = `${url}&state=${res.state}`;
        if (hasMore) {
            await sleep();
        }
    }
    return result;
}