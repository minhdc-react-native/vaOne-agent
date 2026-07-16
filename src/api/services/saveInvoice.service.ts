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
export const saveInvoiceService = {
    async apiToken(data: IInfoLogin) {
        try {
            const res: any = await invoke("http_post", {
                url: "https://login.saveinvoice.vn/api/users/api-token",
                body: data
            });
            const token = res.token;
            const id = await saveInvoiceService.getExternalAccountsIdByUserName(token, data);

            return { token, id } as ILoginRequest;

        } catch (e) {
            let errorMessage = "Đã xảy ra lỗi. Vui lòng kiểm tra lại!";
            if (axios.isAxiosError(e)) {
                errorMessage =
                    e.response?.data?.message ??
                    e.message;
            }
            await dialog.error(errorMessage);
            return null;
        }
    },
    async getExternalAccountsIdByUserName(
        token: string,
        data: IInfoLogin
    ) {
        const res: any = await invoke("http_get", {
            url: "https://login.saveinvoice.vn/api/external_accounts",
            headers: {
                apiToken: token
            },
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
        const id = Array.isArray(res.items)
            ? res.items.find((item: any) => item.username === data.taxCode)?._id ?? null
            : null;
        return id;
    }
}