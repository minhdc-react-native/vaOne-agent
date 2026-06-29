import { useLocation } from "react-router-dom";
import LoginTctPage from "./LoginTctPage";
import LoginMInvoicePage from "./LoginMInvoicePage";

export default function LoginPage() {
    const location = useLocation();
    const params = location.state;
    const source = params.source;
    switch (source) {
        case "TCT": //TCT
            return <LoginTctPage params={params} />
        case "M-INVOICE": //M-Invoice
            return <LoginMInvoicePage params={params} />
        default:
            return <LoginTctPage params={params} />
    }
}