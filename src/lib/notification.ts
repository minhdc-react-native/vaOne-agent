import type { ReactNode } from "react";

type NotificationType = "success" | "error" | "info" | "warning";

const notificationIcons: Record<NotificationType, string> = {
    success: `<svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>`,
    error: `<svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>`,
    info: `<svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>`,
    warning: `<svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>`,
};

const notificationTypeClasses: Record<
    NotificationType,
    { wrapper: string; icon: string }
> = {
    success: {
        wrapper: "bg-green-100 border-green-400",
        icon: "text-green-500",
    },
    error: {
        wrapper: "bg-red-100 border-red-400",
        icon: "text-red-500",
    },
    info: {
        wrapper: "bg-blue-100 border-blue-400",
        icon: "text-blue-500",
    },
    warning: {
        wrapper: "bg-yellow-100 border-yellow-400",
        icon: "text-yellow-500",
    },
};

export const showNotification = ({
    type,
    message,
    duration = 5000,
}: {
    type: NotificationType;
    message: string;
    duration?: number;
}) => {
    const container = document.getElementById("notification-container");
    if (!container) return;

    const notification = document.createElement("div");
    const classes = notificationTypeClasses[type];

    notification.className = `relative w-full p-4 border-l-4 rounded-lg shadow-lg flex items-start gap-3 z-[2147483647] ${classes.wrapper} animate-slideIn`;
    notification.setAttribute("role", "alert");

    notification.innerHTML = `
        <div class="shrink-0 ${classes.icon}" aria-hidden="true">
            ${notificationIcons[type]}
        </div>

        <div class="grow min-w-0 text-sm text-slate-700 pr-4 wrap-break-word">
            ${message}
        </div>

        <button
            class="notification-close absolute top-1.5 right-2 text-slate-400 hover:text-slate-600 text-2xl leading-none"
            aria-label="Đóng"
        >
            &times;
        </button>
    `;

    container.appendChild(notification);

    const closeButton = notification.querySelector(
        ".notification-close"
    ) as HTMLButtonElement;

    let isClosing = false;
    const closeNotification = () => {
        if (isClosing) return;
        isClosing = true;

        clearTimeout(timeoutId);
        notification.classList.remove("animate-slideIn");
        notification.classList.add("animate-fadeOut");
        notification.addEventListener(
            "animationend",
            () => {
                notification.remove();
            },
            { once: true }
        );
    };

    const timeoutId = setTimeout(closeNotification, duration);

    closeButton.addEventListener("click", closeNotification, { once: true });
};