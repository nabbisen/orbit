//! Japanese catalog (RFC-031). Exhaustive over [`MessageKey`].

use super::MessageKey;

pub fn message(key: MessageKey) -> &'static str {
    use MessageKey::*;
    match key {
        AppTitle => "orbok",
        LocalOnlyBadge => "ローカル専用",
        NavSearch => "検索",
        NavSources => "ソース",
        NavIndexing => "インデックス",
        NavStorage => "ストレージ",
        NavModels => "モデル",
        NavAi => "AI",
        NavSettings => "設定",
        SearchPlaceholder => "ローカル文書を検索...",
        SearchButton => "検索",
        SearchNoSourcesTitle => "検索対象がありません",
        SearchNoSourcesBody => {
            "フォルダーまたはファイルを追加すると、orbok がローカル検索\
             インデックスを作成します。"
        }
        SearchAddSource => "ソースを追加",
        SearchNoResults => "結果が見つかりません",
        SearchKeywordOnlyNotice => {
            "セマンティック検索は利用できません。キーワード検索は使用できます。"
        }
        SourcesTitle => "ソース",
        SourcesEmptyTitle => "ソースが登録されていません",
        SourcesEmptyBody => {
            "orbok に検索を許可するフォルダーまたはファイルを追加してください。\
             orbok がコンピューター全体を自動的にスキャンすることはありません。"
        }
        SourcesAddFolder => "フォルダーを追加",
        SourcesStatusActive => "有効",
        SourcesStatusPaused => "一時停止",
        SourcesStatusMissing => "見つかりません",
        IndexingTitle => "インデックス",
        IndexingIdle => "インデックスは最新です",
        IndexingHealthIndexed => "済み",
        IndexingHealthStale => "要更新",
        IndexingHealthFailed => "失敗",
        IndexingHealthQueued => "待機中",
        StorageTitle => "ストレージ",
        StorageIntro => "orbok の保存内容を確認し、安全にクリーンアップできます。",
        StorageSafeCleanupHeading => "安全なクリーンアップ",
        StorageClearSnippets => "一時スニペットを削除",
        StorageClearSearchCache => "期限切れの検索キャッシュを削除",
        StorageDangerHeading => "危険な操作",
        StorageResetCatalog => "カタログをリセット...",
        StorageResetWarning => {
            "登録済みソースとすべてのインデックスを削除します。\
             元のファイルが削除されることはありません。"
        }
        ModelsTitle => "モデル",
        ModelsEmbeddingRole => "埋め込み",
        ModelsRerankerRole => "リランカー",
        ModelsStatusAvailable => "利用可能",
        ModelsStatusMissing => "未導入",
        ModelsKeywordOnlyHint => {
            "キーワード検索は使用できます。概念的な検索を有効にするには、\
             埋め込みモデルを導入してください。"
        }
        SettingsTitle => "設定",
        SettingsLanguageHeading => "言語",
        SettingsPrivacyHeading => "プライバシー",
        SettingsPrivacyLocalOnly => "文書はこのコンピューター上でのみ処理されます。"
        ,
        SearchModeLabel => "モード",
        SearchModeAuto => "自動",
        SearchModeExact => "完全一致",
        SearchModeConceptual => "意味検索",
        SearchModeFast => "高速",
        BadgeKeyword => "キーワード",
        BadgeSemantic => "セマンティック",
        BadgeFused => "融合",
        WizardTitleNotConfigured => "セマンティック検索の設定",
        WizardTitleFileMissing => "埋め込みモデルが見つかりません",
        WizardTitleValidating => "モデルフォルダを確認中",
        WizardTitleReady => "埋め込みモデルの準備完了",
        WizardBodyNotConfigured => {
            "キーワード検索は利用可能です。意味による検索を使用するには、             このコンピュータにローカルAIモデルが必要です。             ファイルはアップロードされません。"
        }
        WizardBodyFileMissing => {
            "モデルフォルダが指定された場所にありません。             ドライブが切断されたか、ファイルが移動した可能性があります。"
        }
        WizardFilesNeededLabel => "フォルダ内の必要ファイル:",
        WizardDownloadHint => "ダウンロード: huggingface-cli download intfloat/multilingual-e5-small",
        WizardPathInputPlaceholder => "モデルフォルダのパス (例: ~/models/multilingual-e5-small)",
        WizardActionLocate => "モデルフォルダを選択",
        WizardActionValidate => "検証",
        WizardActionUseModel => "このモデルを使用",
        WizardActionContinue => "orbok を開始",
        WizardPathPlaceholder => "フォルダのパス…",
        WizardDownloadAction => "HuggingFaceからダウンロード",
        WizardDownloadProgress => "モデルをダウンロード中…",
        WizardActionSkip => "スキップ — キーワード検索のみ使用",
        WizardPreviousPathLabel => "最後の既知のパス",
        WizardValidationOk => "確認済み",
        WizardValidationFail => "見つかりません",
        WizardReadyBody => "セマンティック検索が利用可能になりました。",
        Cancel => "キャンセル",
        Confirm => "確認",
    }
}
