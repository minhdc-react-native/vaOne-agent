// src/api/tokenService.ts

const ACCESS_TOKEN = "access_token";
const REFRESH_TOKEN = "refresh_token";
const TENANT_ID = "tenant_id";

export const tokenService = {
    getAccessToken() {
        return localStorage.getItem(ACCESS_TOKEN);
    },

    setAccessToken(token: string) {
        localStorage.setItem(ACCESS_TOKEN, token);
    },

    getRefreshToken() {
        return localStorage.getItem(REFRESH_TOKEN);
    },

    setRefreshToken(token: string) {
        localStorage.setItem(REFRESH_TOKEN, token);
    },

    getTenantId() {
        return localStorage.getItem(TENANT_ID);
    },

    setTenantId(tenantId: string) {
        localStorage.setItem(TENANT_ID, tenantId);
    },

    clear() {
        localStorage.removeItem(ACCESS_TOKEN);
        localStorage.removeItem(REFRESH_TOKEN);
        localStorage.removeItem(TENANT_ID);
    },
};