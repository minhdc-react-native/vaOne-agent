

class PopupManager {
    private stack: HTMLElement[] = [];

    register(el: HTMLElement) {
        this.stack.push(el);
    }

    unregister(el: HTMLElement) {
        this.stack = this.stack.filter(item => item !== el);
    }

    isTop(el: HTMLElement) {
        return this.stack[this.stack.length - 1] === el;
    }

    getTop() {
        return this.stack[this.stack.length - 1];
    }

    existPopup() { return this.stack.length > 0 }
}

export const popupManager = new PopupManager();
