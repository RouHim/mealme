use std::collections::HashMap;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use tracing::instrument;

use crate::db;
use crate::error::AppError;
use crate::model::{Meal, MealPatch, NewMeal, NewPlanRequest, Plan, PlanPatch, PlanSummaryItem};
use crate::state::AppState;

#[instrument(skip(state))]
pub async fn list_meals(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Meal>>, AppError> {
    let search = params.get("search").map(String::as_str);
    let meals = db::list_meals(&state.pool, search).await?;
    Ok(Json(meals))
}

#[instrument(skip(state))]
pub async fn get_meal(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Meal>, AppError> {
    let meal = db::find_meal(&state.pool, id).await?;
    Ok(Json(meal))
}

#[instrument(skip(state))]
pub async fn create_meal(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NewMeal>,
) -> Result<(StatusCode, Json<Meal>), AppError> {
    let meal = db::insert_meal(&state.pool, payload).await?;
    Ok((StatusCode::CREATED, Json(meal)))
}

#[instrument(skip(state))]
pub async fn update_meal(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(payload): Json<MealPatch>,
) -> Result<Json<Meal>, AppError> {
    let meal = db::update_meal(&state.pool, id, payload).await?;
    Ok(Json(meal))
}

#[instrument(skip(state))]
pub async fn delete_meal(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    db::delete_meal(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Plan handlers
// ---------------------------------------------------------------------------

#[derive(serde::Serialize)]
#[serde(untagged)]
pub enum PlansResponse {
    Single(Plan),
    List(Vec<PlanSummaryItem>),
}

#[instrument(skip(state))]
pub async fn create_plan(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NewPlanRequest>,
) -> Result<(StatusCode, Json<Plan>), AppError> {
    let plan = db::create_or_replace_plan(&state.pool, payload).await?;
    Ok((StatusCode::CREATED, Json(plan)))
}
#[instrument(skip(state))]
pub async fn get_plans(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<PlansResponse>, AppError> {
    let year_str = params
        .get("year")
        .ok_or_else(|| AppError::BadRequest("year is required".into()))?;
    let year: i32 = year_str
        .parse()
        .map_err(|_| AppError::BadRequest("year must be an integer".into()))?;

    if let Some(week_str) = params.get("week") {
        let week: i32 = week_str
            .parse()
            .map_err(|_| AppError::BadRequest("week must be an integer".into()))?;
        let plan = db::get_plan(&state.pool, year, week).await?;
        Ok(Json(PlansResponse::Single(plan)))
    } else {
        let plans = db::list_plans_for_year(&state.pool, year).await?;
        Ok(Json(PlansResponse::List(plans)))
    }
}

#[instrument(skip(state))]
pub async fn update_plan(
    State(state): State<Arc<AppState>>,
    Path((year, week)): Path<(i32, i32)>,
    Json(payload): Json<PlanPatch>,
) -> Result<Json<Plan>, AppError> {
    let plan = db::update_plan_meals(&state.pool, year, week, payload).await?;
    Ok(Json(plan))
}

#[instrument(skip(state))]
pub async fn delete_plan(
    State(state): State<Arc<AppState>>,
    Path((year, week)): Path<(i32, i32)>,
) -> Result<StatusCode, AppError> {
    db::delete_plan(&state.pool, year, week).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::Router;
    use axum::body::to_bytes;
    use axum::http::{Method, Request, StatusCode};
    use axum::routing::{get, put};
    use serde_json::json;

    use tower::ServiceExt;

    use super::*;
    use crate::db::init_db;

    struct TestCtx {
        app: Router,
        _dir: tempfile::TempDir,
    }

    async fn setup() -> TestCtx {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("test.db");
        let pool = init_db(&db_path).await.expect("init_db");
        let state = Arc::new(AppState { pool });
        let app = Router::new()
            .route("/meals", get(list_meals).post(create_meal))
            .route(
                "/meals/:id",
                get(get_meal).put(update_meal).delete(delete_meal),
            )
            .route("/plans", get(get_plans).post(create_plan))
            .route("/plans/:year/:week", put(update_plan).delete(delete_plan))
            .with_state(state);
        TestCtx { app, _dir: dir }
    }

    fn make_ingredient_lines(ings: &[(&str, Option<&str>)]) -> Vec<serde_json::Value> {
        ings.iter()
            .map(|(n, q)| {
                let mut obj = serde_json::Map::new();
                obj.insert("name".into(), json!(n));
                obj.insert("quantity".into(), json!(q));
                serde_json::Value::Object(obj)
            })
            .collect()
    }

    async fn create_meal_helper(
        ctx: &TestCtx,
        name: &str,
        ingredients: &[(&str, Option<&str>)],
    ) -> Meal {
        let body = serde_json::to_vec(&json!({
            "name": name,
            "ingredients": make_ingredient_lines(ingredients)
        }))
        .unwrap();
        let response = ctx
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/meals")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    // -----------------------------------------------------------------------
    // Meal route tests (updated for structured ingredients)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn given_no_meals_when_get_meals_then_returns_200_and_empty_array() {
        let ctx = setup().await;
        let response = ctx
            .app
            .oneshot(
                Request::builder()
                    .uri("/meals")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let meals: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
        assert!(meals.is_empty());
    }

    #[tokio::test]
    async fn given_valid_payload_when_post_meals_then_returns_201_and_persists() {
        let ctx = setup().await;
        let body = serde_json::to_vec(&json!({
            "name": "Pasta",
            "ingredients": make_ingredient_lines(&[("noodles", None), ("sauce", None)])
        }))
        .unwrap();
        let response = ctx
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/meals")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let meal: Meal = serde_json::from_slice(&body).unwrap();
        assert_eq!(meal.name, "Pasta");
        assert_eq!(meal.ingredients.len(), 2);
        assert!(meal.id > 0);
    }

    #[tokio::test]
    async fn given_empty_name_when_post_meals_then_returns_400_with_error() {
        let ctx = setup().await;
        let body = serde_json::to_vec(&json!({
            "name": "",
            "ingredients": make_ingredient_lines(&[("x", None)])
        }))
        .unwrap();
        let response = ctx
            .app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/meals")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["error"].as_str().unwrap().contains("name"));
    }

    #[tokio::test]
    async fn given_existing_meal_when_put_meal_then_returns_200_with_updated_payload() {
        let ctx = setup().await;
        let meal = create_meal_helper(&ctx, "Original", &[("stuff", None)]).await;
        let body = serde_json::to_vec(&json!({
            "name": "Updated",
            "ingredients": make_ingredient_lines(&[("new stuff", None)])
        }))
        .unwrap();
        let response = ctx
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri(format!("/meals/{}", meal.id))
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let updated: Meal = serde_json::from_slice(&body).unwrap();
        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.ingredients.len(), 1);
        assert_eq!(updated.ingredients[0].name, "new stuff");
        assert_eq!(updated.id, meal.id);
    }

    #[tokio::test]
    async fn given_missing_meal_when_put_meal_then_returns_404() {
        let ctx = setup().await;
        let body = serde_json::to_vec(&json!({
            "name": "X",
            "ingredients": make_ingredient_lines(&[("y", None)])
        }))
        .unwrap();
        let response = ctx
            .app
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri("/meals/999")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn given_existing_meal_when_delete_meal_then_returns_204_and_removes_row() {
        let ctx = setup().await;
        let meal = create_meal_helper(&ctx, "ToDelete", &[("x", None)]).await;
        let response = ctx
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri(format!("/meals/{}", meal.id))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let get_resp = ctx
            .app
            .oneshot(
                Request::builder()
                    .uri(format!("/meals/{}", meal.id))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn given_missing_meal_when_delete_meal_then_returns_404() {
        let ctx = setup().await;
        let response = ctx
            .app
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri("/meals/999")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn given_search_term_when_get_meals_then_filters_by_name_and_ingredients() {
        let ctx = setup().await;
        let _ = create_meal_helper(&ctx, "Test", &[("stuff", None)]).await;
        let _ = create_meal_helper(&ctx, "Other", &[("test ingredient", None)]).await;

        let response = ctx
            .app
            .oneshot(
                Request::builder()
                    .uri("/meals?search=test")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let meals: Vec<Meal> = serde_json::from_slice(&body).unwrap();
        assert_eq!(meals.len(), 2);
    }

    // -----------------------------------------------------------------------
    // Plan route tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn given_no_meals_exist_when_post_plans_then_returns_400() {
        let ctx = setup().await;
        let body = serde_json::to_vec(&json!({
            "year": 2026,
            "week_number": 1,
            "meal_count": 3
        }))
        .unwrap();
        let response = ctx
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/plans")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn given_meals_exist_when_post_plans_then_returns_201_with_plan_and_ingredient_summary() {
        let ctx = setup().await;
        create_meal_helper(&ctx, "A", &[("salt", Some("200g"))]).await;
        create_meal_helper(&ctx, "B", &[("salt", Some("100g"))]).await;
        create_meal_helper(&ctx, "C", &[("pepper", None)]).await;

        let body = serde_json::to_vec(&json!({
            "year": 2026,
            "week_number": 1,
            "meal_count": 2
        }))
        .unwrap();
        let response = ctx
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/plans")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let plan: Plan = serde_json::from_slice(&body).unwrap();
        assert_eq!(plan.meals.len(), 2);
        assert!(!plan.ingredient_summary.is_empty());
    }

    #[tokio::test]
    async fn given_plan_exists_when_get_plans_with_year_and_week_then_returns_plan_with_ingredient_summary()
     {
        let ctx = setup().await;
        create_meal_helper(&ctx, "A", &[("salt", Some("200g"))]).await;
        create_meal_helper(&ctx, "B", &[("salt", Some("100g"))]).await;

        // Create a plan
        let body = serde_json::to_vec(&json!({
            "year": 2026,
            "week_number": 1,
            "meal_count": 2
        }))
        .unwrap();
        ctx.app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/plans")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        let response = ctx
            .app
            .oneshot(
                Request::builder()
                    .uri("/plans?year=2026&week=1")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let plan: Plan = serde_json::from_slice(&body).unwrap();
        assert_eq!(plan.meals.len(), 2);
        assert!(!plan.ingredient_summary.is_empty());
    }

    #[tokio::test]
    async fn given_plan_missing_when_get_plans_with_year_and_week_then_returns_404() {
        let ctx = setup().await;
        let response = ctx
            .app
            .oneshot(
                Request::builder()
                    .uri("/plans?year=2026&week=99")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn given_year_query_missing_when_get_plans_then_returns_400() {
        let ctx = setup().await;
        let response = ctx
            .app
            .oneshot(
                Request::builder()
                    .uri("/plans")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn given_year_query_invalid_when_get_plans_then_returns_400() {
        let ctx = setup().await;
        let response = ctx
            .app
            .oneshot(
                Request::builder()
                    .uri("/plans?year=abc")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn given_year_query_only_no_week_when_get_plans_then_returns_summary_array_for_that_year()
    {
        let ctx = setup().await;
        create_meal_helper(&ctx, "A", &[("x", None)]).await;

        // Create a plan
        let body = serde_json::to_vec(&json!({
            "year": 2026,
            "week_number": 1,
            "meal_count": 1
        }))
        .unwrap();
        ctx.app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/plans")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        let response = ctx
            .app
            .oneshot(
                Request::builder()
                    .uri("/plans?year=2026")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let list: Vec<PlanSummaryItem> = serde_json::from_slice(&body).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].week_number, 1);
    }

    #[tokio::test]
    async fn given_plan_exists_when_put_plans_with_meal_ids_then_returns_updated_plan_without_touching_last_planned_at()
     {
        let ctx = setup().await;
        let m1 = create_meal_helper(&ctx, "M1", &[("x", None)]).await;
        let m2 = create_meal_helper(&ctx, "M2", &[("y", None)]).await;

        // Create a plan with m1, m2 via POST
        let body = serde_json::to_vec(&json!({
            "year": 2026,
            "week_number": 1,
            "meal_count": 2
        }))
        .unwrap();
        ctx.app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/plans")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Record last_planned_at values after generation
        let get_resp = ctx
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/meals/{}", m1.id))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = to_bytes(get_resp.into_body(), 4096).await.unwrap();
        let m1_after: Meal = serde_json::from_slice(&body).unwrap();
        let lp1 = m1_after.last_planned_at;

        // Replace plan with just m2 via PUT
        let put_body = serde_json::to_vec(&json!({
            "meal_ids": [m2.id]
        }))
        .unwrap();
        let put_resp = ctx
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri("/plans/2026/1")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(put_body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(put_resp.status(), StatusCode::OK);

        // Verify m1's last_planned_at unchanged
        let get_resp2 = ctx
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/meals/{}", m1.id))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body2 = to_bytes(get_resp2.into_body(), 4096).await.unwrap();
        let m1_final: Meal = serde_json::from_slice(&body2).unwrap();
        assert_eq!(m1_final.last_planned_at, lp1);
    }

    #[tokio::test]
    async fn given_plan_missing_when_put_plans_then_returns_404() {
        let ctx = setup().await;
        let body = serde_json::to_vec(&json!({
            "meal_ids": [1]
        }))
        .unwrap();
        let response = ctx
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri("/plans/2026/99")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn given_plan_exists_when_delete_plans_then_returns_204_and_subsequent_get_returns_404() {
        let ctx = setup().await;
        create_meal_helper(&ctx, "A", &[("x", None)]).await;

        // Create a plan
        let body = serde_json::to_vec(&json!({
            "year": 2026,
            "week_number": 1,
            "meal_count": 1
        }))
        .unwrap();
        ctx.app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/plans")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Delete
        let del_resp = ctx
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri("/plans/2026/1")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(del_resp.status(), StatusCode::NO_CONTENT);

        // Verify gone
        let get_resp = ctx
            .app
            .oneshot(
                Request::builder()
                    .uri("/plans?year=2026&week=1")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn given_plan_missing_when_delete_plans_then_returns_404() {
        let ctx = setup().await;
        let response = ctx
            .app
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri("/plans/2026/99")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
