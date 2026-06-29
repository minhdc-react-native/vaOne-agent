import AppWindow from "../components/AppWindow";
import Button from "../components/Button";
interface IProgs {
    params: Record<string, any>
}
export default function LoginMInvoicePage({ params }: IProgs) {
    return (
        <AppWindow title="Đăng nhập">
            <div className="flex h-full flex-col gap-4 p-4">

                <div className="mb-2 text-center">
                    <h2 className="text-xl font-bold">
                        M-Invoice
                    </h2>

                    <p className="text-sm text-gray-500">
                        Đăng nhập Hóa đơn điện tử
                    </p>
                </div>

                <Button
                    className="mt-2 w-full"
                    onClick={() => { }}
                >
                    Đăng nhập
                </Button>
            </div>
        </AppWindow>
    );
}