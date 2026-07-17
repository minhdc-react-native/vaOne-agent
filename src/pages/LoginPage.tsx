import { useLocation } from "react-router-dom";
import LoginTctPage from "./LoginTctPage";
import LoginMInvoicePage from "./LoginMInvoicePage";
import LoginSaveInvoicePage from "./LoginSaveInvoicePage";
import { useAppStore } from "../stores/app.store";
import { useEffect } from "react";
import { hideWindow } from "../components/AppWindow";
import LoginEmpty from "./LoginEmpty";

export default function LoginPage() {
    const login = useAppStore(s => s.login);
    const location = useLocation();
    console.log("location>>", location);
    const params = location.state || login;
    const source = params?.source;

    switch (source) {
        case "TCT": //TCT
            return <LoginTctPage params={params} />
        case "M-SMI": //M-SMI
            return <LoginMInvoicePage params={params} />
        case "SAVE-INVOICE": //Save-Invoice
            return <LoginSaveInvoicePage params={params} />
        default:
            return <LoginEmpty />
    }
}