pub mod draft_order_loader;
pub mod draft_order_validator;
pub mod grade_generator;
pub mod loader;
pub mod position_mapper;
pub mod rankings_loader;
pub mod rankings_validator;
pub mod scouting_report_loader;
pub mod scouting_report_validator;
pub mod team_loader;
pub mod team_need_loader;
pub mod team_need_validator;
pub mod team_season_loader;
pub mod team_season_validator;
pub mod team_validator;
pub mod validator;

/// Standard number of rounds in an NFL draft
pub const NFL_DRAFT_ROUNDS: i32 = 7;

/// Maximum allowed round number (generous upper bound for validation)
pub const MAX_DRAFT_ROUND: i32 = 7;

/// Compensatory picks are only allowed in these rounds (inclusive)
pub const COMPENSATORY_ROUND_MIN: i32 = 3;
pub const COMPENSATORY_ROUND_MAX: i32 = 7;
