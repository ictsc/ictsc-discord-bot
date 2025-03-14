use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Team {
    /// チーム番号
    pub id: String,
    /// チームロール名（かつ、チーム名）
    pub role_name: String,
    /// 招待コード
    pub invitation_code: String,
    pub user_group_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Problem {
    /// 問題コード (i.e. "ABC", "DEF")
    pub code: String,
    /// 問題タイトル
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Contestant {
    /// ユーザー名、ユニーク
    pub name: String,
    /// ユーザー表示名
    pub display_name: String,
    /// 所属チームのコード
    pub team_id: String,
    /// Discord ID
    pub discord_id: String,
}
