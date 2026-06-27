import { authService } from "../services/auth.service";
import { tokenService } from "../tokenService";

export async function refreshTokenRequest() {
    const refreshToken = tokenService.getRefreshToken();

    if (!refreshToken) {
        throw new Error(
            "No refresh token"
        );
    }

    const response = await authService.refreshToken(refreshToken);

    const {
        access_token,
        refresh_token,
    } = response.data;

    if (access_token) {
        tokenService.setAccessToken(
            access_token
        );
    }

    if (refresh_token) {
        tokenService.setRefreshToken(
            refresh_token
        );
    }

    return access_token;
}