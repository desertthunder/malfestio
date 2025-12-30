//! Spaced Repetition System (SM-2 Algorithm)
//!
//! Implements the SuperMemo 2 algorithm for scheduling card reviews.
//! Parameters are designed to be user-configurable in the future.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Grade given by user during review (0-5 scale)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Grade(pub u8);

impl Grade {
    pub const AGAIN: Grade = Grade(0);
    pub const HARD: Grade = Grade(1);
    pub const GOOD: Grade = Grade(3);
    pub const EASY: Grade = Grade(4);
    pub const PERFECT: Grade = Grade(5);

    pub fn new(value: u8) -> Option<Self> {
        if value <= 5 { Some(Grade(value)) } else { None }
    }

    pub fn is_passing(&self) -> bool {
        self.0 >= 3
    }
}

/// Default SM-2 parameters (user-configurable in future)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sm2Config {
    /// Initial ease factor for new cards
    pub initial_ease: f32,
    /// Minimum ease factor (prevents cards from becoming too hard)
    pub min_ease: f32,
    /// First interval in days after initial correct answer
    pub first_interval: i32,
    /// Second interval in days
    pub second_interval: i32,
}

impl Default for Sm2Config {
    fn default() -> Self {
        Self { initial_ease: 2.5, min_ease: 1.3, first_interval: 1, second_interval: 6 }
    }
}

/// Current review state for a card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewState {
    /// Ease factor (multiplier for interval)
    pub ease_factor: f32,
    /// Current interval in days
    pub interval_days: i32,
    /// Number of consecutive correct reviews
    pub repetitions: i32,
    /// When the card is due
    pub due_at: DateTime<Utc>,
}

impl Default for ReviewState {
    fn default() -> Self {
        Self { ease_factor: 2.5, interval_days: 0, repetitions: 0, due_at: Utc::now() }
    }
}

impl ReviewState {
    /// Create a new review state for a fresh card
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate next review state based on grade using SM-2 algorithm
    pub fn schedule(&self, grade: Grade, config: &Sm2Config) -> Self {
        let q = grade.0 as f32;

        // TODO: move to separate fn
        // EF' = EF + (0.1 - (5 - q) * (0.08 + (5 - q) * 0.02))
        let new_ease = self.ease_factor + (0.1 - (5.0 - q) * (0.08 + (5.0 - q) * 0.02));
        let new_ease = new_ease.max(config.min_ease);

        if grade.is_passing() {
            let (new_interval, new_reps) = match self.repetitions {
                0 => (config.first_interval, 1),
                1 => (config.second_interval, 2),
                _ => {
                    let interval = (self.interval_days as f32 * new_ease).round() as i32;
                    (interval.max(1), self.repetitions + 1)
                }
            };

            Self {
                ease_factor: new_ease,
                interval_days: new_interval,
                repetitions: new_reps,
                due_at: Utc::now() + Duration::days(new_interval as i64),
            }
        } else {
            Self { ease_factor: new_ease, interval_days: 0, repetitions: 0, due_at: Utc::now() + Duration::minutes(10) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grade_validation() {
        assert!(Grade::new(0).is_some());
        assert!(Grade::new(5).is_some());
        assert!(Grade::new(6).is_none());
    }

    #[test]
    fn test_grade_passing() {
        assert!(!Grade::AGAIN.is_passing());
        assert!(!Grade::HARD.is_passing());
        assert!(Grade::GOOD.is_passing());
        assert!(Grade::EASY.is_passing());
        assert!(Grade::PERFECT.is_passing());
    }

    #[test]
    fn test_new_card_first_review_correct() {
        let config = Sm2Config::default();
        let state = ReviewState::new();

        let next = state.schedule(Grade::GOOD, &config);

        assert_eq!(next.interval_days, 1);
        assert_eq!(next.repetitions, 1);
        assert!(next.ease_factor >= 2.3 && next.ease_factor <= 2.7);
    }

    #[test]
    fn test_new_card_first_review_incorrect() {
        let config = Sm2Config::default();
        let state = ReviewState::new();

        let next = state.schedule(Grade::AGAIN, &config);

        assert_eq!(next.interval_days, 0);
        assert_eq!(next.repetitions, 0);
        let diff = next.due_at - Utc::now();
        assert!(diff.num_minutes() <= 15);
    }

    #[test]
    fn test_second_review_correct() {
        let config = Sm2Config::default();
        let state = ReviewState { ease_factor: 2.5, interval_days: 1, repetitions: 1, due_at: Utc::now() };
        let next = state.schedule(Grade::GOOD, &config);

        assert_eq!(next.interval_days, 6);
        assert_eq!(next.repetitions, 2);
    }

    #[test]
    fn test_mature_card_interval_grows() {
        let config = Sm2Config::default();
        let state = ReviewState { ease_factor: 2.5, interval_days: 10, repetitions: 5, due_at: Utc::now() };
        let next = state.schedule(Grade::GOOD, &config);

        assert!(next.interval_days >= 20);
        assert_eq!(next.repetitions, 6);
    }

    #[test]
    fn test_ease_factor_minimum() {
        let config = Sm2Config::default();
        let state = ReviewState { ease_factor: 1.4, interval_days: 5, repetitions: 3, due_at: Utc::now() };
        let next = state.schedule(Grade::GOOD, &config);

        assert!(next.ease_factor >= config.min_ease);
    }

    #[test]
    fn test_easy_increases_ease() {
        let config = Sm2Config::default();
        let state = ReviewState::new();

        let next = state.schedule(Grade::PERFECT, &config);

        assert!(next.ease_factor > 2.5);
    }

    #[test]
    fn test_hard_decreases_ease() {
        let config = Sm2Config::default();
        let state = ReviewState { ease_factor: 2.5, interval_days: 10, repetitions: 5, due_at: Utc::now() };
        let next = state.schedule(Grade::HARD, &config);

        assert!(next.ease_factor < 2.5);
    }

    #[test]
    fn test_30_day_simulation() {
        let config = Sm2Config::default();
        let mut state = ReviewState::new();

        state = state.schedule(Grade::GOOD, &config);
        assert_eq!(state.interval_days, 1);

        state = state.schedule(Grade::GOOD, &config);
        assert_eq!(state.interval_days, 6);

        state = state.schedule(Grade::GOOD, &config);
        assert!(state.interval_days >= 12 && state.interval_days <= 20);

        state = state.schedule(Grade::GOOD, &config);
        assert!(state.interval_days >= 20);
        assert_eq!(state.repetitions, 4);
        assert!(state.ease_factor >= config.min_ease);
    }
}
