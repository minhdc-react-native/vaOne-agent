import axios from "axios";

export async function imageUrlToBase64(url: string): Promise<string> {
    try {
        const response = await axios.get(url, {
            responseType: "blob",
        });

        console.log("image response:", {
            url,
            status: response.status,
            contentType: response.headers["content-type"],
            size: response.data.size,
        });

        return await new Promise<string>((resolve, reject) => {
            const reader = new FileReader();

            reader.onload = () => {
                if (typeof reader.result === "string") {
                    resolve(reader.result);
                } else {
                    reject(new Error("Không thể convert image sang base64"));
                }
            };

            reader.onerror = () => {
                reject(reader.error ?? new Error("FileReader error"));
            };

            reader.readAsDataURL(response.data);
        });
    } catch (error) {
        console.error("imageUrlToBase64 error:", url, error);
        throw error;
    }
}