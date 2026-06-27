// src/api/axios/axiosClient.ts
import axios, { type AxiosInstance, type AxiosRequestConfig } from "axios";
import { tokenService } from "../tokenService";
import { refreshTokenRequest } from "./refreshToken";

let refreshPromise: Promise<string | null> | null = null;

const axiosClient = axios.create({
    baseURL: "http://10.8.0.146:30334",
    timeout: 30000
});

export const trayApi = axios.create({
    baseURL: "http://127.0.0.1:15682",
    timeout: 30000,
});

// REQUEST

axiosClient.interceptors.request.use(
    (config) => {
        const token = tokenService.getAccessToken();
        const tenantId = tokenService.getTenantId();
        if (token) {
            config.headers.Authorization = `Bearer ${token}`;
        }
        if (tenantId) {
            config.headers.__tenant = tenantId;
        }
        config.headers["Accept-Language"] = "vi";
        return config;
    },
    (error) => Promise.reject(error)
);

// RESPONSE

axiosClient.interceptors.response.use(
    (response) => {
        if (response.config.responseType === "blob" || response.config.responseType === "arraybuffer") {
            return response;
        }
        return response.data;
    },

    async (error) => {

        const originalRequest = error.config as CustomAxiosRequestConfig;

        if (!error.response) {
            return Promise.reject(error);
        }

        if (error.response.status !== 401) {
            return Promise.reject(error);
        }

        if (originalRequest._retry) {
            return Promise.reject(error);
        }

        originalRequest._retry = true;

        try {
            if (!refreshPromise) {
                refreshPromise = refreshTokenRequest().finally(() => {
                    refreshPromise = null;
                });
            }

            const newToken = await refreshPromise;
            if (!newToken) {
                throw new Error(
                    "Refresh token failed"
                );
            }
            originalRequest.headers = originalRequest.headers || {};

            originalRequest.headers.Authorization = `Bearer ${newToken}`;

            return axiosClient(originalRequest);
        } catch (err) {
            tokenService.clear();
            return Promise.reject(err);
        }
    }
);
interface CustomAxiosRequestConfig extends AxiosRequestConfig {
    _retry?: boolean;
}

export interface ApiClient extends AxiosInstance {
    get<T = any>(
        url: string,
        config?: CustomAxiosRequestConfig
    ): Promise<T>;

    post<T = any>(
        url: string,
        data?: any,
        config?: CustomAxiosRequestConfig
    ): Promise<T>;

    put<T = any>(
        url: string,
        data?: any,
        config?: CustomAxiosRequestConfig
    ): Promise<T>;

    delete<T = any>(
        url: string,
        config?: CustomAxiosRequestConfig
    ): Promise<T>;
}

export default axiosClient as ApiClient;
