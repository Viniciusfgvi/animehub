// src-tauri/src/integrations/anilist/client.rs
//
// AniList API Integration - Phase 5 Real Implementation
//
// ARCHITECTURE:
// - GraphQL client for AniList API
// - Handles authentication, rate limiting, pagination
// - Maps external data → internal DTOs (NO domain mutation)
// - Used by ExternalIntegrationService
//
// CRITICAL RULES:
// - This is INFRASTRUCTURE, not DOMAIN
// - Never creates or modifies domain entities directly
// - Returns DTOs that services can map
// - Handles all external API concerns

use crate::error::{AppError, AppResult};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// AniList anime metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListAnime {
    pub id: i64,
    pub title: AniListTitle,
    pub episodes: Option<i32>,
    pub status: String,
    pub start_date: Option<AniListDate>,
    pub end_date: Option<AniListDate>,
    pub genres: Vec<String>,
    pub cover_image: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListDate {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

/// GraphQL response wrapper
#[derive(Debug, Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
    #[allow(dead_code)] // Part of GraphQL error response schema
    status: Option<i32>,
}

/// Search results wrapper
#[derive(Debug, Deserialize)]
struct SearchData {
    #[serde(rename = "Page")]
    page: PageData,
}

#[derive(Debug, Deserialize)]
struct PageData {
    media: Vec<MediaData>,
}

/// Single anime query wrapper
#[derive(Debug, Deserialize)]
struct AnimeData {
    #[serde(rename = "Media")]
    media: MediaData,
}

/// Media data from AniList
#[derive(Debug, Deserialize)]
struct MediaData {
    id: i64,
    title: TitleData,
    episodes: Option<i32>,
    status: String,
    #[serde(rename = "startDate")]
    start_date: Option<DateData>,
    #[serde(rename = "endDate")]
    end_date: Option<DateData>,
    genres: Vec<String>,
    #[serde(rename = "coverImage")]
    cover_image: CoverImageData,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TitleData {
    romaji: Option<String>,
    english: Option<String>,
    native: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DateData {
    year: Option<i32>,
    month: Option<i32>,
    day: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct CoverImageData {
    large: String,
}

/// Rate limiter state
struct RateLimiter {
    last_request: Instant,
    min_interval: Duration,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            last_request: Instant::now() - Duration::from_secs(60),
            min_interval: Duration::from_millis(1000), // 1 request per second
        }
    }

    fn wait_if_needed(&mut self) {
        let elapsed = self.last_request.elapsed();
        if elapsed < self.min_interval {
            let wait_time = self.min_interval - elapsed;
            std::thread::sleep(wait_time);
        }
        self.last_request = Instant::now();
    }
}

/// AniList API Client
pub struct AniListClient {
    base_url: String,
    http_client: Client,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    auth_token: Option<String>,
}

impl AniListClient {
    /// Create a new AniList client
    pub fn new() -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url: "https://graphql.anilist.co".to_string(),
            http_client,
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new())),
            auth_token: None,
        }
    }

    /// Create client with authentication token
    pub fn with_auth(token: String) -> Self {
        let mut client = Self::new();
        client.auth_token = Some(token);
        client
    }

    /// Search for anime by title
    ///
    /// Returns up to 10 results
    pub async fn search_anime(&self, query: &str) -> AppResult<Vec<AniListAnime>> {
        // Rate limiting
        {
            let mut limiter = self.rate_limiter.lock().unwrap();
            limiter.wait_if_needed();
        }

        // GraphQL query
        let graphql_query = r#"
            query ($search: String) {
                Page(page: 1, perPage: 10) {
                    media(search: $search, type: ANIME) {
                        id
                        title {
                            romaji
                            english
                            native
                        }
                        episodes
                        status
                        startDate {
                            year
                            month
                            day
                        }
                        endDate {
                            year
                            month
                            day
                        }
                        genres
                        coverImage {
                            large
                        }
                        description
                    }
                }
            }
        "#;

        let variables = json!({
            "search": query
        });

        let response_data: SearchData = self.execute_query(graphql_query, variables).await?;

        // Map to AniListAnime
        let animes = response_data
            .page
            .media
            .into_iter()
            .map(Self::map_media_to_anime)
            .collect();

        Ok(animes)
    }

    /// Get anime by AniList ID
    pub async fn get_anime(&self, anilist_id: i64) -> AppResult<AniListAnime> {
        // Rate limiting
        {
            let mut limiter = self.rate_limiter.lock().unwrap();
            limiter.wait_if_needed();
        }

        // GraphQL query
        let graphql_query = r#"
            query ($id: Int) {
                Media(id: $id, type: ANIME) {
                    id
                    title {
                        romaji
                        english
                        native
                    }
                    episodes
                    status
                    startDate {
                        year
                        month
                        day
                    }
                    endDate {
                        year
                        month
                        day
                    }
                    genres
                    coverImage {
                        large
                    }
                    description
                }
            }
        "#;

        let variables = json!({
            "id": anilist_id
        });

        let response_data: AnimeData = self.execute_query(graphql_query, variables).await?;

        Ok(Self::map_media_to_anime(response_data.media))
    }

    /// Fetch detailed metadata (same as get_anime for now)
    pub async fn fetch_metadata(&self, anilist_id: i64) -> AppResult<AniListAnime> {
        self.get_anime(anilist_id).await
    }

    // ========================================================================
    // INTERNAL: GraphQL Execution
    // ========================================================================

    /// Execute a GraphQL query
    async fn execute_query<T>(&self, query: &str, variables: serde_json::Value) -> AppResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let body = json!({
            "query": query,
            "variables": variables
        });

        // Build request
        let mut request = self
            .http_client
            .post(&self.base_url)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCEPT, "application/json");

        // Add auth token if present
        if let Some(token) = &self.auth_token {
            request = request.header(header::AUTHORIZATION, format!("Bearer {}", token));
        }

        // Send request
        let response = request
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Other(format!("AniList API request failed: {}", e)))?;

        // Check HTTP status
        if !response.status().is_success() {
            return Err(AppError::Other(format!(
                "AniList API returned status: {}",
                response.status()
            )));
        }

        // Parse GraphQL response
        let graphql_response: GraphQLResponse<T> = response
            .json()
            .await
            .map_err(|e| AppError::Other(format!("Failed to parse AniList response: {}", e)))?;

        // Check for GraphQL errors
        if let Some(errors) = graphql_response.errors {
            let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();

            return Err(AppError::Other(format!(
                "AniList API errors: {}",
                error_messages.join(", ")
            )));
        }

        // Extract data
        graphql_response
            .data
            .ok_or_else(|| AppError::Other("AniList API returned no data".to_string()))
    }

    /// Map MediaData to AniListAnime
    fn map_media_to_anime(media: MediaData) -> AniListAnime {
        AniListAnime {
            id: media.id,
            title: AniListTitle {
                romaji: media.title.romaji,
                english: media.title.english,
                native: media.title.native,
            },
            episodes: media.episodes,
            status: media.status,
            start_date: media.start_date.map(|d| AniListDate {
                year: d.year,
                month: d.month,
                day: d.day,
            }),
            end_date: media.end_date.map(|d| AniListDate {
                year: d.year,
                month: d.month,
                day: d.day,
            }),
            genres: media.genres,
            cover_image: media.cover_image.large,
            description: media.description,
        }
    }
}

impl Default for AniListClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = AniListClient::new();
        assert_eq!(client.base_url, "https://graphql.anilist.co");
        assert!(client.auth_token.is_none());
    }

    #[test]
    fn test_client_with_auth() {
        let client = AniListClient::with_auth("test_token".to_string());
        assert!(client.auth_token.is_some());
    }

    // Note: Real API tests would be in integration test suite
    // and would use mocked responses or test against real API
}
