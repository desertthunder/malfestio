use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum PreferencesRepoError {
    DatabaseError(String),
    NotFound(String),
}

/// User persona for personalized experience
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Persona {
    Learner,
    Creator,
    Curator,
}

impl std::fmt::Display for Persona {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Persona::Learner => write!(f, "learner"),
            Persona::Creator => write!(f, "creator"),
            Persona::Curator => write!(f, "curator"),
        }
    }
}

impl std::str::FromStr for Persona {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "learner" => Ok(Persona::Learner),
            "creator" => Ok(Persona::Creator),
            "curator" => Ok(Persona::Curator),
            _ => Err(format!("Invalid persona: {}", s)),
        }
    }
}

/// User preferences for onboarding and personalization
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserPreferences {
    #[serde(default)]
    pub user_did: String,
    pub persona: Option<Persona>,
    pub onboarding_completed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub tutorial_deck_completed: bool,
    pub density_mode: Option<String>,
}

/// Update request for user preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePreferences {
    pub persona: Option<Persona>,
    pub complete_onboarding: Option<bool>,
    pub tutorial_deck_completed: Option<bool>,
    pub density_mode: Option<String>,
}

#[async_trait]
pub trait PreferencesRepository: Send + Sync {
    /// Get user preferences, creating default if not exists
    async fn get_or_create(&self, user_did: &str) -> Result<UserPreferences, PreferencesRepoError>;

    /// Update user preferences
    async fn update(&self, user_did: &str, updates: UpdatePreferences)
    -> Result<UserPreferences, PreferencesRepoError>;
}

pub struct DbPreferencesRepository {
    pool: crate::db::DbPool,
}

impl DbPreferencesRepository {
    pub fn new(pool: crate::db::DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PreferencesRepository for DbPreferencesRepository {
    async fn get_or_create(&self, user_did: &str) -> Result<UserPreferences, PreferencesRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| PreferencesRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // Try to get existing preferences
        let row = client
            .query_opt(
                "SELECT user_did, persona, onboarding_completed_at, tutorial_deck_completed, density_mode FROM user_prefs WHERE user_did = $1",
                &[&user_did],
            )
            .await
            .map_err(|e| PreferencesRepoError::DatabaseError(format!("Failed to query preferences: {}", e)))?;

        if let Some(row) = row {
            let persona_str: Option<String> = row.get("persona");
            let persona = persona_str.and_then(|s| s.parse().ok());

            return Ok(UserPreferences {
                user_did: row.get("user_did"),
                persona,
                onboarding_completed_at: row.get("onboarding_completed_at"),
                tutorial_deck_completed: row.get("tutorial_deck_completed"),
                density_mode: row.get("density_mode"),
            });
        }

        // Create default preferences
        client
            .execute(
                "INSERT INTO user_prefs (id, user_did) VALUES ($1, $2) ON CONFLICT (user_did) DO NOTHING",
                &[&uuid::Uuid::new_v4(), &user_did],
            )
            .await
            .map_err(|e| PreferencesRepoError::DatabaseError(format!("Failed to create preferences: {}", e)))?;

        Ok(UserPreferences { user_did: user_did.to_string(), ..Default::default() })
    }

    async fn update(
        &self, user_did: &str, updates: UpdatePreferences,
    ) -> Result<UserPreferences, PreferencesRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| PreferencesRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // Ensure record exists first
        client
            .execute(
                "INSERT INTO user_prefs (id, user_did) VALUES ($1, $2) ON CONFLICT (user_did) DO NOTHING",
                &[&uuid::Uuid::new_v4(), &user_did],
            )
            .await
            .map_err(|e| PreferencesRepoError::DatabaseError(format!("Failed to ensure preferences: {}", e)))?;

        // Build update query dynamically
        let mut set_clauses = Vec::new();
        let mut param_idx = 2;

        let persona_str = updates.persona.map(|p| p.to_string());
        if updates.persona.is_some() {
            set_clauses.push(format!("persona = ${}", param_idx));
            param_idx += 1;
        }

        let now = Utc::now();
        let complete_onboarding = updates.complete_onboarding.unwrap_or(false);

        if updates.tutorial_deck_completed.is_some() {
            set_clauses.push(format!("tutorial_deck_completed = ${}", param_idx));
            param_idx += 1;
        }

        if updates.density_mode.is_some() {
            set_clauses.push(format!("density_mode = ${}", param_idx));
            param_idx += 1;
        }

        if complete_onboarding {
            set_clauses.push(format!("onboarding_completed_at = ${}", param_idx));
        }

        if set_clauses.is_empty() {
            return self.get_or_create(user_did).await;
        }

        // Build params list - need to handle owned values
        let mut param_vec: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
        param_vec.push(Box::new(user_did.to_string()));

        if let Some(ref persona) = persona_str {
            param_vec.push(Box::new(persona.clone()));
        }

        if let Some(tutorial) = updates.tutorial_deck_completed {
            param_vec.push(Box::new(tutorial));
        }

        if let Some(ref density) = updates.density_mode {
            param_vec.push(Box::new(density.clone()));
        }

        if complete_onboarding {
            param_vec.push(Box::new(now));
        }

        let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = param_vec
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let query = format!("UPDATE user_prefs SET {} WHERE user_did = $1", set_clauses.join(", "));

        client
            .execute(&query, &params_refs)
            .await
            .map_err(|e| PreferencesRepoError::DatabaseError(format!("Failed to update preferences: {}", e)))?;

        self.get_or_create(user_did).await
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    pub struct MockPreferencesRepository {
        pub prefs: Arc<Mutex<std::collections::HashMap<String, UserPreferences>>>,
        pub should_fail: Arc<Mutex<bool>>,
    }

    impl MockPreferencesRepository {
        pub fn new() -> Self {
            Self {
                prefs: Arc::new(Mutex::new(std::collections::HashMap::new())),
                should_fail: Arc::new(Mutex::new(false)),
            }
        }

        #[allow(dead_code)]
        pub fn set_should_fail(&self, should_fail: bool) {
            *self.should_fail.lock().unwrap() = should_fail;
        }
    }

    impl Default for MockPreferencesRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl PreferencesRepository for MockPreferencesRepository {
        async fn get_or_create(&self, user_did: &str) -> Result<UserPreferences, PreferencesRepoError> {
            if *self.should_fail.lock().unwrap() {
                return Err(PreferencesRepoError::DatabaseError("Mock failure".to_string()));
            }

            let mut prefs = self.prefs.lock().unwrap();
            let entry = prefs
                .entry(user_did.to_string())
                .or_insert_with(|| UserPreferences { user_did: user_did.to_string(), ..Default::default() });
            Ok(entry.clone())
        }

        async fn update(
            &self, user_did: &str, updates: UpdatePreferences,
        ) -> Result<UserPreferences, PreferencesRepoError> {
            if *self.should_fail.lock().unwrap() {
                return Err(PreferencesRepoError::DatabaseError("Mock failure".to_string()));
            }

            let mut prefs = self.prefs.lock().unwrap();
            let entry = prefs
                .entry(user_did.to_string())
                .or_insert_with(|| UserPreferences { user_did: user_did.to_string(), ..Default::default() });

            if let Some(persona) = updates.persona {
                entry.persona = Some(persona);
            }

            if updates.complete_onboarding.unwrap_or(false) {
                entry.onboarding_completed_at = Some(Utc::now());
            }

            if let Some(tutorial) = updates.tutorial_deck_completed {
                entry.tutorial_deck_completed = tutorial;
            }

            if let Some(density) = updates.density_mode {
                entry.density_mode = Some(density);
            }

            Ok(entry.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::MockPreferencesRepository;
    use super::*;

    #[tokio::test]
    async fn test_get_or_create_returns_default() {
        let repo = MockPreferencesRepository::new();
        let prefs = repo.get_or_create("did:plc:test").await.unwrap();

        assert_eq!(prefs.user_did, "did:plc:test");
        assert!(prefs.persona.is_none());
        assert!(prefs.onboarding_completed_at.is_none());
        assert!(!prefs.tutorial_deck_completed);
    }

    #[tokio::test]
    async fn test_update_persona() {
        let repo = MockPreferencesRepository::new();
        let prefs = repo
            .update(
                "did:plc:test",
                UpdatePreferences {
                    persona: Some(Persona::Creator),
                    complete_onboarding: None,
                    tutorial_deck_completed: None,
                    density_mode: None,
                },
            )
            .await
            .unwrap();

        assert_eq!(prefs.persona, Some(Persona::Creator));
    }

    #[tokio::test]
    async fn test_complete_onboarding() {
        let repo = MockPreferencesRepository::new();
        let prefs = repo
            .update(
                "did:plc:test",
                UpdatePreferences {
                    persona: Some(Persona::Learner),
                    complete_onboarding: Some(true),
                    tutorial_deck_completed: None,
                    density_mode: None,
                },
            )
            .await
            .unwrap();

        assert!(prefs.onboarding_completed_at.is_some());
        assert_eq!(prefs.persona, Some(Persona::Learner));
    }

    #[tokio::test]
    async fn test_persona_parse() {
        assert_eq!("learner".parse::<Persona>().unwrap(), Persona::Learner);
        assert_eq!("creator".parse::<Persona>().unwrap(), Persona::Creator);
        assert_eq!("curator".parse::<Persona>().unwrap(), Persona::Curator);
        assert!("invalid".parse::<Persona>().is_err());
    }
}
