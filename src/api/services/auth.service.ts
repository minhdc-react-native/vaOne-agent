
import axios from "axios";
const CLIENT_ID = "VaOneApi_Web";
const CLIENT_SECRET = "1q2w3e*";
// const SCOPE = "VaOneApi offline_access";
interface ITokenResponse {
    name: string;
    access_token: string;
    refresh_token: string;
    expires_in: number;
    token_type: string;
    scope: string;
}

export const authService = {
    async refreshToken(refreshToken: string) {
        const body = new URLSearchParams({
            client_id: CLIENT_ID,
            client_secret: CLIENT_SECRET,
            grant_type: "refresh_token",
            refresh_token: refreshToken,
            // scope: SCOPE,
        });

        return axios.post<ITokenResponse>(
            `${import.meta.env.VITE_API_URL}/connect/token`,
            body,
            {
                headers: {
                    "Content-Type": "application/x-www-form-urlencoded",
                },
            }
        );
    },
};