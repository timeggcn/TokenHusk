//! 质量反馈 IPC 命令（submit_feedback）。

use tokenhusk_core::observation::models::Feedback;
use tokenhusk_core::observation::recorder::Recorder;

/// 提交质量反馈（👍/👎）。
#[tauri::command]
pub fn submit_feedback(
    request_id: u64,
    thumbs_up: bool,
    comment: Option<String>,
) -> bool {
    let feedback = Feedback {
        id: 0,
        request_id,
        thumbs_up,
        comment,
        created_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
    };
    Recorder::record_feedback(feedback);
    true
}
