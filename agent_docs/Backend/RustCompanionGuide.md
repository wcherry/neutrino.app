# Rust Backend Patterns Guide

Purpose

This document defines implementation patterns for backend development in Rust.

It complements the Backend Architecture Development Guide by providing:
	•	concrete design patterns
	•	implementation conventions
	•	examples for developers
	•	performance-aware practices

These patterns are intended to produce backend services that are:
	•	fast
	•	predictable
	•	maintainable
	•	modular

⸻

1. Backend Layer Architecture

Backend services should follow a four-layer structure.

API Layer
Application Layer
Domain Layer
Infrastructure Layer

Each layer has a clear responsibility.

⸻

2. API Layer

Responsibility

The API layer handles:
	•	HTTP requests
	•	input validation
	•	authentication
	•	request/response mapping

It must not contain business logic.

⸻

Recommended Libraries

Minimal framework approach:
	•	Axum or Hyper
	•	Serde for serialization
	•	Tower middleware

Avoid large frameworks that hide execution flow.

⸻

Handler Structure

Handlers should remain thin.

Example structure:

pub async fn get_user(
    Path(id): Path<UserId>,
    State(service): State<UserService>,
) -> Result<Json<UserResponse>, ApiError> {
    let user = service.get_user(id).await?;
    Ok(Json(user.into()))
}

Rules:

Handlers should only:
	•	validate request
	•	call service
	•	return response

⸻

Request Validation

Validation should occur at the API boundary.

Example:

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub name: String,
}

Validation can be implemented with:
	•	manual checks
	•	validation crates
	•	custom domain types

⸻

3. Application Layer (Service Layer)

Responsibility

The service layer contains application logic and orchestrates domain operations.

Responsibilities:
	•	coordinate domain entities
	•	enforce workflows
	•	manage transactions
	•	call repositories

Example:

pub struct UserService<R: UserRepository> {
    repo: R,
}

Example method:

pub async fn create_user(
    &self,
    input: CreateUserInput
) -> Result<User> {
    let user = User::new(input)?;
    self.repo.save(&user).await?;
    Ok(user)
}

Rules:
	•	services should be stateless
	•	services should depend on traits
	•	services should not know database details

⸻

4. Domain Layer

The domain layer represents business logic and rules.

It includes:
	•	entities
	•	value objects
	•	domain services
	•	domain errors

The domain layer must be independent of infrastructure.

⸻

Entities

Entities represent core business objects.

Example:

pub struct User {
    pub id: UserId,
    pub email: Email,
    pub name: String,
}

Entities may contain logic:

impl User {
    pub fn change_email(&mut self, email: Email) {
        self.email = email;
    }
}

Avoid anemic models.

⸻

Value Objects

Use value types to enforce invariants.

Example:

pub struct Email(String);

Constructor:

impl Email {
    pub fn new(value: String) -> Result<Self> {
        if !value.contains("@") {
            return Err(Error::InvalidEmail);
        }

        Ok(Self(value))
    }
}

Benefits:
	•	validation occurs once
	•	prevents invalid states

⸻

5. Repository Pattern

Repositories isolate persistence logic.

They are defined as traits in the domain or service layer.

Example:

#[async_trait]
pub trait UserRepository {
    async fn find(&self, id: UserId) -> Result<Option<User>>;
    async fn save(&self, user: &User) -> Result<()>;
}

Concrete implementations live in infrastructure.

Example:

infrastructure/
    database/
        user_repository.rs


⸻

Example Implementation

pub struct PostgresUserRepository {
    pool: PgPool,
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find(&self, id: UserId) -> Result<Option<User>> {
        // SQL here
    }
}

Rules:
	•	SQL must live only in repository implementations
	•	services never run queries directly

⸻

6. Error Handling Pattern

All modules should define typed errors.

Example:

#[derive(thiserror::Error, Debug)]
pub enum UserError {
    #[error("user not found")]
    NotFound,

    #[error("email already exists")]
    EmailExists,
}

Service errors should be mapped to API errors.

Example mapping:

Domain Error → Service Error → API Error

Avoid exposing internal errors to clients.

⸻

7. Async vs Sync Design

Rust backends should use async carefully.

Use async for:
	•	database calls
	•	network requests
	•	file IO

Avoid async for:
	•	pure CPU logic
	•	simple in-memory operations

Async has overhead.

Use it only where beneficial.

⸻

8. Background Worker Pattern

Background jobs should run in separate workers.

Examples:
	•	feed refresh
	•	scraping
	•	email sending
	•	analytics

Worker design:

worker/
   job.rs
   scheduler.rs

Example job trait:

pub trait Job {
    async fn run(&self) -> Result<()>;
}

Workers must be:
	•	idempotent
	•	retryable
	•	observable

⸻

9. Transaction Pattern

Transactions should live in the service layer.

Example:

pub async fn create_order(
    &self,
    input: CreateOrderInput
) -> Result<Order> {
    let mut tx = self.repo.begin().await?;

    let order = Order::new(input)?;
    self.repo.save_tx(&mut tx, &order).await?;

    tx.commit().await?;

    Ok(order)
}

Rules:
	•	transactions should be short-lived
	•	avoid network calls inside transactions

⸻

10. Dependency Injection

Rust favors explicit dependency injection.

Avoid global state.

Prefer constructor injection.

Example:

pub struct UserService<R> {
    repo: R,
}

Initialization:

let repo = PostgresUserRepository::new(pool);
let service = UserService::new(repo);

This keeps dependencies visible.

⸻

11. Serialization Pattern

Separate API DTOs from domain models.

Example:

api/
    dto.rs

domain/
    models.rs

Example DTO:

pub struct UserResponse {
    id: String,
    email: String,
}

Conversion:

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            email: user.email.to_string(),
        }
    }
}

Benefits:
	•	domain remains stable
	•	API can evolve independently

⸻

12. Configuration Pattern

Use a typed configuration object.

Example:

pub struct Config {
    pub database_url: String,
    pub http_port: u16,
}

Load configuration at startup only.

Never read environment variables deep inside modules.

⸻

13. Logging Pattern

Use structured logging.

Example:

tracing::info!(
    user_id = %user_id,
    action = "login",
    "user logged in"
);

Avoid string-only logs.

Always include context fields.

⸻

14. Metrics Pattern

Expose metrics for:
	•	request latency
	•	DB query duration
	•	job execution time
	•	queue depth

Example metrics:

api_request_duration
db_query_duration
worker_jobs_processed

Metrics should be cheap to collect.

⸻

15. Performance Guidelines

Critical backend rules:

Avoid unnecessary cloning

Bad:

let x = value.clone();

Prefer borrowing.

⸻

Prefer slices

&[T]

instead of

Vec<T>

when ownership is not needed.

⸻

Avoid unnecessary allocations

Use iterators and references.

⸻

Use streaming when possible

Example:
	•	large file downloads
	•	large query results

⸻

16. Testing Patterns

Unit Tests

Test domain logic independently.

Example:

#[test]
fn email_validation() {
    assert!(Email::new("bad".into()).is_err());
}


⸻

Integration Tests

Test real infrastructure.

Example:

tests/
    api_tests.rs
    repository_tests.rs

Use isolated databases.

⸻

17. Migrations

Database schema must be versioned.

Use migration tools such as:
	•	sqlx migrations
	•	refinery
	•	goose

Rules:
	•	migrations must be immutable
	•	schema changes must be backward compatible

⸻

18. Recommended Crates

Suggested minimal backend ecosystem:

Core:

tokio
axum
serde
serde_json

Database:

sqlx

Utilities:

thiserror
anyhow
tracing
uuid
chrono

Testing:

tokio-test
reqwest


⸻

19. Anti-Patterns to Avoid

Avoid the following patterns.

⸻

God Services

Bad:

AppService

Split by domain.

⸻

Database Everywhere

Business logic should not contain SQL.

⸻

Hidden Dependencies

Avoid global state.

⸻

Excessive Framework Abstraction

Prefer explicit code over magical frameworks.

⸻

20. Engineering Philosophy

Rust backends should aim for:
	•	predictable performance
	•	explicit architecture
	•	minimal dependencies
	•	clear module boundaries

Rust enables systems that are:
	•	fast
	•	safe
	•	maintainable

But only if developers embrace explicit design rather than hiding complexity behind frameworks.
