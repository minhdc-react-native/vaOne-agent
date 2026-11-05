// src/api/axios/axiosClient.ts
import axios from "axios";

export const trayApi = axios.create({
    baseURL: "http://127.0.0.1:15682",
    timeout: 30000,
});
