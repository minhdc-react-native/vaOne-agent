import { invoke } from "@tauri-apps/api/core";
import { useEffect } from "react";

export const BlankPage = () => {
    useEffect(() => {
        invoke("page_ready", { name: "blank", show: false });
    }, []);
    return null;
}