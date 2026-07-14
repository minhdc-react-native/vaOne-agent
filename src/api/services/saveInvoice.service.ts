import axios from "axios";
import { invoke } from "@tauri-apps/api/core";
import { dialog } from "../../service/dialog.service";
interface IInfoLogin {
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
            const id = await saveInvoiceService.getExternalAccountsFirstId(token);

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
    },
    async getExternalAccountsFirstId(
        token: string
    ) {
        try {

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
            const id = Array.isArray(res.items) && res.items.length > 0
                ? res.items[0]._id
                : null;
            return id;
        } catch (e) {
            await dialog.error("Không thể lấy mã Captcha!");
            return null;
        }
    }
}