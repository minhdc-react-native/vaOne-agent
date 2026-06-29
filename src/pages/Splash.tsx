export const SplashScreen = () => {
    return (<div className="fixed inset-0 z-50 flex flex-col items-center justify-center bg-white">
        {/* Logo */}
        <img
            src="/LogoVacom.png"
            alt="splash"
            className="w-68 h-22 object-contain"
        />
    </div>);
}