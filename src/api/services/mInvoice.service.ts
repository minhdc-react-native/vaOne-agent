import axios from "axios";
import { invoke } from "@tauri-apps/api/core";
import { dialog } from "../../service/dialog.service";
interface IInfoLogin {
    taxCode: string;
    username: string;
    password: string;
}
interface ILoginRequest {
    token: string;
    id: string;
}
export const mInvoiceService = {
    async login(data: IInfoLogin) {
        const res: any = await invoke("http_post", {
            url: "https://qlhd.minvoice.com.vn/api/users/login",
            body: data
        });
        return res._id;
    },
    async apiToken(data: IInfoLogin) {
        try {
            const id = await mInvoiceService.login(data);

            const res: any = await invoke("http_get", {
                url: "https://qlhd.minvoice.com.vn/api/users/api_tokens",
                params: {
                    lazyLoadEvent: JSON.stringify({
                        first: 0,
                        rows: 20,
                        page: 0,
                        sortField: null,
                        sortOrder: null,
                        filters: {
                            tags: {
                                value: null,
                                matchMode: "in"
                            }
                        }
                    })
                }
            });
            const token = Array.isArray(res.items)
                ? res.items.find((item: any) => item.user === id)?.token ?? null
                : null;

            return { token, id } as ILoginRequest;

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
    }
}