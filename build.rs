fn main() {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(
            &[
                "protos/hook.proto",
                "protos/vhook.proto"
            ],
            &[] as &[&str],
        )
        .unwrap();
}