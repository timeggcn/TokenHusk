// 防 Windows 在 release 下弹出多余 console；dev 下不影响。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tokenhusk::run()
}
