#!/usr/bin/env bash
# TokenHusk 开发环境 PATH / LIB 配置（本机 FlyEnv 管理的工具链，机器相关）。
# 用法：source scripts/env.sh   （或 . scripts/env.sh）
#
# 说明：
#   - FlyEnv 把 Node 18 / Rust 1.95 分开存放，且 Rust 的 rust-std 未合并进 sysroot，
#     cargo 直接运行会报 `can't find crate for std`。
#     项目内 `.cargo/config.toml` 已把 sysroot 指向 `.toolchain/sysroot` 解决；
#     这里只需保证 cargo / rustc / node 在 PATH 即可。
#   - LIB 指向 Windows SDK 的导入库（FlyEnv 无 VS Build Tools 的 VC 目录，
#     链接器用 rust-lld，见 `.cargo/config.toml`）。

# Node 18.20.8（FlyEnv）
export PATH="/d/sofeware/FlyEnv-Data/app/nodejs/v18.20.8:$PATH"
# Rust：rustc shim（env/rust/bin）+ cargo（app/rust/1.95.0/cargo/bin）
export PATH="/d/sofeware/FlyEnv-Data/env/rust/bin:/d/sofeware/FlyEnv-Data/app/rust/1.95.0/cargo/bin:$PATH"

# Windows SDK 导入库（lld-link 用）
SDK_LIB="C:/Program Files (x86)/Windows Kits/10/Lib/10.0.26100.0"
export LIB="${SDK_LIB}/um/x64;${SDK_LIB}/ucrt/x64"

echo "env.sh loaded: $(command -v cargo) $(command -v node)"
