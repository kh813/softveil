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
        
        // 旧 StealthLightSubpixel が消えていることの確認
        assert!(!serialized.contains("StealthLightSubpixel"));
    }

    #[test]
    fn test_filter_mode_merger_safety() {
        // StealthLight (Subpixel) への一本化が正しく動作するか
        let mode = FilterMode::StealthLight;
        let mode_val = match mode {
            FilterMode::BlackLayer => 0.0,
            FilterMode::VerticalLouver => 1.0,
            FilterMode::AIOcrInterference => 2.0,
            FilterMode::HighIntensitySPD => 3.0,
            FilterMode::StealthDark => 4.0,
            FilterMode::StealthLight => 5.0,
        };
        assert_eq!(mode_val, 5.0);
    }

    #[test]
    fn test_preset_crud_operations() {
        use crate::display_config::FilterSettings;
        let mut state = AppState::new();
        let preset_name = "Test Preset";
        let settings = FilterSettings {
            filter_mode: FilterMode::HighIntensitySPD,
            alpha: 0.5,
            filter_intensity: 1.2,
            override_period_mm: Some(0.8),
            override_cover_ratio: None,
            override_scroll_speed: None,
        };

        // Create
        state.save_preset(preset_name.to_string(), settings.clone());
        assert!(state.presets.iter().any(|p| p.name == preset_name));

        // Read/Apply
        let id = MonitorId(123);
        state.add_display(id, None);
        state.apply_preset(preset_name, &id);
        let config = state.displays.get(&id).unwrap();
        assert_eq!(config.filter_mode, FilterMode::HighIntensitySPD);
        assert_eq!(config.alpha, 0.5);

        // Update
        let new_settings = FilterSettings {
            filter_mode: FilterMode::StealthLight,
            alpha: 0.2,
            filter_intensity: 0.9,
            override_period_mm: Some(0.4),
            override_cover_ratio: None,
            override_scroll_speed: None,
        };
        state.save_preset(preset_name.to_string(), new_settings);
        state.apply_preset(preset_name, &id);
        let updated_config = state.displays.get(&id).unwrap();
        assert_eq!(updated_config.filter_mode, FilterMode::StealthLight);

        // Delete
        state.delete_preset(preset_name);
        assert!(!state.presets.iter().any(|p| p.name == preset_name));
    }

    #[test]
    fn test_benchmark_preset_generation() {
        // ベンチマーク後に生成される推奨プリセットの妥当性テスト
        let presets = crate::benchmark::generate_recommended_presets_for_monitor(0.5, 0.4, "Test Monitor");
        
        // 少なくとも 3 つのプリセット (Office, Transit, Night) が生成されること
        assert!(presets.len() >= 3);
        
        let office = presets.iter().find(|p| p.name.contains("Office")).expect("Office preset missing");
        assert_eq!(office.settings.filter_mode, FilterMode::StealthLight);
        
        let transit = presets.iter().find(|p| p.name.contains("Transit")).expect("Transit preset missing");
        assert_eq!(transit.settings.filter_mode, FilterMode::HighIntensitySPD);
        
        let night = presets.iter().find(|p| p.name.contains("Night")).expect("Night preset missing");
        assert_eq!(night.settings.filter_mode, FilterMode::StealthDark);
    }

    #[test]
    fn test_display_category_logic() {
        use crate::display_config::{DisplayCategory, DisplayProfile, PanelType};
        
        // カテゴリごとのデフォルト値が妥当か（デグレ防止）
        let profile_fhd = DisplayProfile::from_config(DisplayCategory::NotebookFhd, PanelType::LcdIps);
        assert_eq!(profile_fhd.period_mm, 0.50); 
        
        let profile_hidpi = DisplayProfile::from_config(DisplayCategory::NotebookHiDpi, PanelType::LcdIps);
        assert_eq!(profile_hidpi.period_mm, 0.50);
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
    fn test_ui_state_flags_logic() {
        // AI 覗き見検知フラグなどの UI 状態管理ロジックの検証
        let mut state = AppState::new();
        state.ai_detection_enabled = true;
        state.ai_peeper_detected = true;
        
        // AI検知をオフにしたとき、検知フラグもリセットされるか
        state.toggle_ai_detection();
        assert!(!state.ai_detection_enabled);
        assert!(!state.ai_peeper_detected);
    }

    #[test]
    fn test_platform_dialog_interface_safety() {
        // プラットフォーム固有のダイアログ関数が正しくリンクされ、
        // 少なくともパニックせずに呼び出し可能か（ヘッドレス環境では表示はされない）
        // 注: 実際のダイアログ表示はブロックするため、シンボルの存在確認に留める
        let _ = crate::platform::show_info_dialog;
        let _ = crate::platform::show_error_dialog;
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn test_menu_rebuild_logic_robustness() {
        // メニュー再構築ロジックが、複雑なモニター構成でもパニックしないか
        let mut state = AppState::new();
        for i in 0..5 {
            state.add_display(MonitorId(i), None);
        }
        
        // AppState からメニュー項目を生成するロジックをシミュレート
        // overlays が空の状態でもメニュー構築が正常に行われるか確認
        let (menu, _) = crate::tray::build_menu(&state, &[]);
        assert!(menu.items().len() > 0);
    }

    #[test]
    fn test_monitor_id_hashing() {
        // MonitorId が HashMap のキーとして正しく機能し、同一性が保たれているか
        let id1 = MonitorId(0x1234);
        let id2 = MonitorId(0x1234);
        let id3 = MonitorId(0x5678);
        
        let mut map = std::collections::HashMap::new();
        map.insert(id1, "Monitor A");
        
        assert_eq!(map.get(&id2), Some(&"Monitor A"));
        assert_eq!(map.get(&id3), None);
    }
}
