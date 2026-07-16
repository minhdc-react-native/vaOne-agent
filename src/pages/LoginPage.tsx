import { useLocation } from "react-router-dom";
import LoginTctPage from "./LoginTctPage";
import LoginMInvoicePage from "./LoginMInvoicePage";
import LoginSaveInvoicePage from "./LoginSaveInvoicePage";
import { useAppStore } from "../stores/app.store";

export default function LoginPage() {
    const login = useAppStore(s => s.login);
    const location = useLocation();
    const params = location.state || login;
    const source = params.source;
    switch (source) {
        case "TCT": //TCT
            return <LoginTctPage params={params} />
        case "M-INVOICE": //M-Invoice
            return <LoginMInvoicePage params={params} />
        case "SAVE-INVOICE": //Save-Invoice
            return <LoginSaveInvoicePage params={params} />
        default:
            return <LoginTctPage params={params} />
    }
}