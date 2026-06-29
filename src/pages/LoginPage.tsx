import { useLocation } from "react-router-dom";
import LoginTctPage from "./LoginTctPage";
import LoginMInvoicePage from "./LoginMInvoicePage";

export default function LoginPage() {
    const location = useLocation();
    const params = location.state;
    const source = params.source as number;
    switch (source) {
        case 1: //TCT
            return <LoginTctPage params={params} />
        case 2: //M-Invoice
            return <LoginMInvoicePage params={params} />
        default:
            return <LoginTctPage params={params} />
    }
}