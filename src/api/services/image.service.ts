import axios from "axios";

export async function imageUrlToBase64(url: string): Promise<string> {
    const response = await axios.get(url, {
        responseType: "blob",
    });

    return await new Promise((resolve, reject) => {
        const reader = new FileReader();

        reader.onloadend = () => resolve(reader.result as string);

        reader.onerror = reject;

        reader.readAsDataURL(response.data);
    });
}