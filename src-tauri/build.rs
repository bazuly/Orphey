fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new().app_manifest(
            tauri_build::AppManifest::new().commands(&[
                "bootstrap",
                "list_requests",
                "create_room",
                "delete_room",
                "set_room_base_url",
                "delete_request",
                "send_request",
                "run_load",
            ]),
        ),
    )
    .expect("failed to run tauri-build");
}
