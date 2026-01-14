cargo build --target wasm32-unknown-unknown --release
deploy -e --file-path AuditLayer/target/wasm32-unknown-unknown/release/AuditLayer.wasm --widl-file AuditLayer/auditlayer.widl