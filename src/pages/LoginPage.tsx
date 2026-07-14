import { useLocation } from "react-router-dom";
import LoginTctPage from "./LoginTctPage";
import LoginMInvoicePage from "./LoginMInvoicePage";
import LoginSaveInvoicePage from "./LoginSaveInvoicePage";

export default function LoginPage() {
    const location = useLocation();
    const params = location.state;
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