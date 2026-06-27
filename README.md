# Tauri + React + Typescript

This template should help get you started developing with Tauri, React and Typescript in Vite.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

services/
│
├── auth_service.rs
├── printer_service.rs
├── download_service.rs
├── pdf_service.rs
├── notification_service.rs
└── local_server/
      ├── handlers.rs
      ├── routes.rs
      ├── middleware.rs
      └── mod.rs


#tạo icons:
yarn tauri icon src-tauri/icons/1024x1024.png      

#build:

rm -rf src-tauri/target
yarn tauri build