//! Japanese catalog (RFC-031). Exhaustive over [`MessageKey`].

use super::MessageKey;

pub fn message(key: MessageKey) -> &'static str {
    use MessageKey::*;
    match key {
        AppTitle => "orbit",
        LocalOnlyBadge => "ローカル専用",
        NavSearch => "検索",
        NavSources => "ソース",
        NavIndexing => "インデックス",
        NavStorage => "ストレージ",
        NavModels => "モデル",
        NavSettings => "設定",
        SearchPlaceholder => "ローカル文書を検索...",
        SearchButton => "検索",
        SearchNoSourcesTitle => "検索対象がありません",
        SearchNoSourcesBody => {
            "フォルダーまたはファイルを追加すると、orbit がローカル検索\
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
            "orbit に検索を許可するフォルダーまたはファイルを追加してください。\
             orbit がコンピューター全体を自動的にスキャンすることはありません。"
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
        StorageIntro => "orbit の保存内容を確認し、安全にクリーンアップできます。",
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
        Cancel => "キャンセル",
        Confirm => "確認",
    }
}
