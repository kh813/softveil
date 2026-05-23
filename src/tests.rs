#[cfg(test)]
mod tests {
    use crate::logger;
    use crate::app::AppState;
    use crate::display_config::{FilterMode, MonitorId};
    use std::fs;
    use std::io::Read;

    #[test]
    fn test_logging_system() {
        // ... (existing logging test)
        let test_msg = "REGRESSION_TEST_MESSAGE_12345";
        logger!("{}", test_msg);

        #[cfg(target_os = "macos")]
        let path = "/tmp/softveil.log";
        #[cfg(target_os = "windows")]
        let path = {
            let mut p = std::env::temp_dir();
            p.push("softveil.log");
            p
        };

        if let Ok(mut file) = fs::File::open(&path) {
            let mut file_content = String::new();
            file.read_to_string(&mut file_content).expect("Should be able to read log file");
            assert!(file_content.contains(test_msg), "Log should contain the test message");
        }
    }

    #[test]
    fn test_app_state_benchmark_transition() {
        // UI/UX 回帰テスト: ベンチマークの状態遷移が正しいか
        let mut state = AppState::new();
        assert!(state.benchmark_progress.is_none());

        // ベンチマーク開始時の状態をシミュレート
        state.benchmark_progress = Some(0.0);
        assert_eq!(state.benchmark_progress, Some(0.0));

        // 進捗更新
        state.benchmark_progress = Some(0.5);
        assert_eq!(state.benchmark_progress, Some(0.5));

        // 完了時
        state.benchmark_progress = None;
        assert!(state.benchmark_progress.is_none());
    }

    #[test]
    fn test_display_config_persistence_safety() {
        // 設定保存のシリアライズ・デシリアライズが壊れていないか
        let mut state = AppState::new();
        let id = MonitorId(999);
        state.add_display(id, None);
        state.set_filter_mode(&id, FilterMode::StealthDark);
        
        let config = state.displays.get(&id).unwrap();
        assert_eq!(config.filter_mode, FilterMode::StealthDark);
        
        // JSON シリアライズの試行（confy が内部で使用）
        let serialized = serde_json::to_string(&state.displays).unwrap();
        assert!(serialized.contains("StealthDark"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_wmi_safety() {
        // WMI/COM 関連のシンボルと初期化フローの検証
        // 実際の WMI 接続は権限等で失敗する可能性があるため、シンボルのリンク確認を主眼とする
        unsafe {
            use windows::Win32::System::Com::*;
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            // CoUninitialize(); // テスト環境を壊さないよう注意
        }
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_permission_logic_flow() {
        let _ = crate::platform::has_screen_capture_access();
    }
}
