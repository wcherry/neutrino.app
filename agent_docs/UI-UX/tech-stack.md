# Neutrino UI Tech Stack

1. Core Framework

React

Primary UI framework.

Reasons:
	•	Mature ecosystem
	•	Component architecture aligns with your shared component library
	•	Works for Web + Mobile
	•	Supports strict typing
	•	Large tooling ecosystem

Stack:

React 18+


⸻

Next.js (Web Applications)

Used for the web interface.

Reasons:
	•	SSR + streaming
	•	File-based routing
	•	Built-in performance optimization
	•	Edge deployment compatibility
	•	Good dev experience

Stack:

Next.js 15+
React Server Components
App Router


2. Language

TypeScript

Mandatory across the UI stack.

Reasons:
	•	Strong typing
	•	Better component contracts
	•	Safer refactors
	•	Consistency with Rust backend contracts

TypeScript 5+


⸻

3. Styling System

Your design guide explicitly calls for:
	•	design tokens
	•	shared CSS
	•	minimal dependencies  ￼

Global CSS Token System

Custom CSS variables.

/ui/tokens
/ui/themes

Example:

--color-bg-primary
--color-border
--spacing-md
--radius-sm

These power the design system.

⸻

4. Component Library

Instead of third-party frameworks:

Build an internal component system.

Structure:

/ui/components

Examples:

Button
Input
Select
Checkbox
Card
Modal
Toast
Table
Tabs
Sidebar

Rules:
	•	No third-party UI frameworks
	•	Token-driven styling
	•	Accessible by default

This aligns directly with your component-first rule  ￼.

⸻

5. Data Fetching

TanStack Query

@tanstack/react-query

Reasons:
	•	caching
	•	optimistic updates
	•	background refresh
	•	retry handling

Important for:

API calls
background sync
state synchronization


⸻

6. State Management

Use minimal state libraries.

Primary:

React state
React context

For global state:

Zustand

zustand

Reasons:
	•	tiny
	•	simple
	•	no boilerplate
	•	works well with React

Use for:

auth
preferences
UI state


⸻

7. Forms

React Hook Form

react-hook-form

Reasons:
	•	high performance
	•	minimal rerenders
	•	good TypeScript support

Validation:

zod


⸻

8. Icons

Your guide recommends minimal outline icons  ￼.

Recommended:

Lucide

Advantages:
	•	clean
	•	lightweight
	•	tree-shakable

⸻

9. Motion

Animations should be subtle.

Recommended:

Framer Motion

Use only for:

modals
drawer transitions
micro interactions

Avoid heavy animation systems.

⸻

10. UI Testing

Playwright

You already referenced Playwright previously.

Use for:

E2E testing
component interaction
visual regression


⸻

11. Component Development

Storybook

Optional but recommended.

Purpose:

component testing
design validation
UI documentation


⸻

12. Build Tools

Handled by Next.js, but core stack includes:

Vite (for non-Next tooling)
ESBuild
SWC


⸻

13. Package Management

Recommended:

pnpm

Reasons:
	•	fast
	•	efficient monorepo support

⸻

14. Monorepo Structure

Recommended:

/apps
    /web

/packages
    /ui
    /design-tokens
    /api-client
    /utils


⸻

15. API Client

Generate strongly typed clients.

Recommended:

OpenAPI
or
tRPC

Prefer:

OpenAPI + codegen

Because Rust backend likely exposes OpenAPI spec.

⸻

16. Accessibility

Use:

react-aria

or internal wrappers.

Ensures:

keyboard navigation
ARIA support
screen reader support


⸻

17. Documentation

Store UI documentation in:

/docs/ui

Include:

component standards
design tokens
interaction patterns


⸻

Final Recommended Stack

Language
TypeScript

Web
Next.js
React

Mobile
Native applications - outside initial scope

Styling
CSS Tokens

Components
Internal Component Library

State
Zustand
React Context

Data Fetching
TanStack Query

Forms
React Hook Form
Zod

Icons
Lucide

Animation
Framer Motion

Testing
Playwright

Component Dev
Storybook

Package Manager
pnpm


⸻

Guiding Stack Philosophy

Neutrino UI should follow three rules:

1️⃣ Internal components over UI frameworks

2️⃣ CSS tokens over ad-hoc styling

3️⃣ Small dependencies over large frameworks

This ensures:

long-term maintainability
visual consistency
high performance
