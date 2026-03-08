# Backend Architecture Development Guide

1. Purpose

This document defines the architectural standards and development principles for backend services. The goals are to:
	•	Maintain high performance and low latency
	•	Ensure clear module boundaries and maintainability
	•	Minimize external dependencies and service coupling
	•	Follow idiomatic Rust design principles
	•	Provide well-documented APIs and contracts
	•	Enable long-term scalability and reliability

All backend services must be written in Rust and adhere to these guidelines unless an explicit architectural exception is approved.

⸻

2. Core Architectural Principles

2.1 Performance First

Backend systems must prioritize performance and resource efficiency.

Guidelines:
	•	Prefer zero-cost abstractions
	•	Avoid unnecessary allocations
	•	Minimize locking and shared mutable state
	•	Prefer async I/O for network-bound operations
	•	Prefer synchronous execution for CPU-bound operations

Key principles:
	•	Latency-sensitive paths must avoid unnecessary abstractions
	•	Memory usage should be predictable
	•	Avoid hidden complexity from frameworks

Performance must be considered during:
	•	API design
	•	database access
	•	serialization
	•	background processing

⸻

2.2 Limit External Services

External services introduce:
	•	latency
	•	operational risk
	•	vendor lock-in
	•	cost
	•	failure domains

Therefore:

External services should only be used when they provide clear operational or technical advantage.

Examples where external services may be acceptable:
	•	email delivery
	•	SMS delivery
	•	payment processing
	•	CDN distribution

Examples that should not rely on external services when possible:
	•	queues
	•	caching
	•	authentication
	•	search
	•	storage

Preference order:
	1.	In-process solution
	2.	Self-hosted infrastructure
	3.	External service

⸻

2.3 Modularity

The system must be organized into independent modules, each with a single responsibility.

Each module:
	•	owns its internal logic
	•	owns its internal data structures
	•	exposes functionality only through explicit interfaces

Modules must not depend on internal implementation details of other modules.

Communication must occur through clearly defined contracts.

⸻

3. Rust Standards

All backend code must follow Rust best practices and idioms.

Standards include:

Formatting

Use:

cargo fmt

Formatting rules must never be bypassed.

⸻

Linting

All code must pass:

cargo clippy --all-targets --all-features -D warnings

Warnings must be resolved before merging.

⸻

Error Handling

Prefer explicit error types.

Use:

Result<T, E>

Avoid:

panic!()
unwrap()
expect()

Except in:
	•	tests
	•	startup initialization where failure is fatal

⸻

Dependency Management

Guidelines:
	•	Minimize dependency count
	•	Avoid large framework dependencies
	•	Prefer small, focused crates
	•	Avoid crates with excessive macro usage

Every dependency should answer:

What problem does this solve that we cannot reasonably implement ourselves?

⸻

4. Module Design

4.1 Single Responsibility

Each module must represent one domain concern.

Examples:

auth
users
storage
search
notifications

Avoid modules that combine responsibilities:

Bad:

user_auth_notifications

Good:

users
auth
notifications


⸻

4.2 Module Encapsulation

Modules must hide internal implementation details.

Rust visibility rules must enforce boundaries:

pub(crate)
pub(super)
pub

Prefer the most restrictive visibility possible.

Internal structs should not be exposed publicly unless required.

⸻

4.3 Contracts Between Modules

Modules communicate through well-defined contracts.

Contracts may include:
	•	traits
	•	request/response structs
	•	domain models
	•	service interfaces

Example:

pub trait UserRepository {
    fn find_by_id(&self, id: UserId) -> Result<User>;
}

Modules should depend on traits, not concrete implementations.

⸻

4.4 Dependency Direction

Dependencies should always flow inward toward core logic.

Example:

API Layer
   ↓
Application Layer
   ↓
Domain Layer
   ↓
Infrastructure Layer

Core business logic must not depend on:
	•	HTTP frameworks
	•	database drivers
	•	third-party services

⸻

5. Recommended Project Structure

**Example backend workspace layout:**
/backend
	/drive - drive application
	/sheets - sheets application
	/docs - docs application
	/slides - slide application
	/shared - shared library
	/worker - worker async application

**Example application layout:**
src/
	main.rs	
	migrations/ - diesel migrations scripts (SQL)
	features/
		mod.rs

		storage/
			mod.rs
			api.rs
			service.rs
			dto.rs
			model.rs
			repository.rs

		auth/
			mod.rs
			api.rs
			dto.rs
			service.rs
			tokens.rs
			repository.rs

		usage/
			mod.rs
			api.rs
			service.rs
			models.rs

		shared/       - shared internally between features
			mod.rs

	config/
		mod.rs
		middleware


⸻

6. API Design Standards

All APIs must be explicitly documented.

Documentation must include:
	•	endpoint description
	•	request parameters
	•	response schema
	•	error responses
	•	authentication requirements

⸻

6.1 Endpoint Consistency

Use consistent resource naming.

Example:

GET /users
GET /users/{id}
POST /users
PATCH /users/{id}
DELETE /users/{id}

Avoid verbs in endpoints:

Bad:

POST /createUser

Good:

POST /users


⸻

6.2 Request and Response Models

API structures must be separate from internal domain models.

Example:

api/models.rs
domain/models.rs

This prevents API changes from impacting core logic.

⸻

6.3 Error Responses

All APIs must return structured error responses.

Example:

{
  "error": {
    "code": "USER_NOT_FOUND",
    "message": "User does not exist"
  }
}

Avoid returning raw system errors.

⸻

7. Data Access

Database logic must be isolated in repository layers.

Responsibilities of repositories:
	•	diesel queries
	•	database transactions
	•	persistence mapping

Business logic must not contain SQL.

Example:

users/
  repository.rs
  service.rs


⸻

8. Background Processing

Background tasks must be handled by dedicated workers.

Examples:
	•	feed refresh
	•	scraping
	•	email sending
	•	analytics aggregation

Workers must be:
	•	idempotent
	•	restart-safe
	•	observable

⸻

9. Observability

Every backend service must include:

Logging

Structured logs only.

Example fields:

request_id
user_id
operation
duration_ms
error


⸻

Metrics

Expose metrics for:
	•	request latency
	•	error rate
	•	worker throughput
	•	queue depth
	•	DB query time

⸻

Tracing

Distributed tracing should exist for:
	•	API requests
	•	background jobs
	•	database operations

⸻

10. Testing Standards

Each module must include:

Unit Tests

Testing internal logic without external systems.

cargo test


⸻

Integration Tests

Testing interactions between modules and infrastructure.

Use isolated test databases when required.

⸻

Deterministic Tests

Tests must:
	•	not rely on external services
	•	not depend on time
	•	not require network access

⸻

11. Configuration Management

Configuration must be:
	•	environment driven
	•	validated at startup
	•	strongly typed

Example:

DATABASE_URL
PORT
LOG_LEVEL

Use structured config objects rather than reading environment variables throughout the codebase.

⸻

12. Security Standards

Required practices:
	•	validate all external input
	•	enforce authentication at API boundaries
	•	never log secrets
	•	protect against injection attacks
	•	apply rate limiting to public APIs

Sensitive data must be:
	•	encrypted in transit
	•	minimized at rest

⸻

13. Versioning and Backwards Compatibility

Public APIs must follow versioning.

Example:

/api/v1/users

Breaking changes must require:
	•	new version
	•	migration plan

⸻

14. Code Review Standards

All code must undergo peer review.

Review checklist:
	•	follows module boundaries
	•	performance implications considered
	•	minimal dependencies added
	•	error handling implemented
	•	tests included
	•	documentation updated

⸻

15. Guiding Philosophy

The backend should prioritize:
	1.	Clarity over cleverness
	2.	Performance over abstraction
	3.	Modularity over convenience
	4.	Reliability over speed of development

The system should remain understandable by a new engineer within days, not months.

16. Misc
	1. Use actix macros to define endpoints
	2. Use service configuration to create routes
	```	pub fn configure(conf: &mut web::ServiceConfig) {
	    	conf.service(
        		web::scope("/storage")
            		.service(create_file)
					...
	```

