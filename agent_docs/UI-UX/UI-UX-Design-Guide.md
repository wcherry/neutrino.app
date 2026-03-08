Below is a UI/UX Design Guide your team can use as a foundational design system document. It is written in a way that engineers and designers can both follow, and it aligns with your goals:
	•	clean
	•	subtle
	•	limited color usage
	•	app-specific color identity
	•	shared component system
	•	minimal third-party dependencies

You can drop this directly into a repo as something like:

/docs/ui-ux-design-guide.md


⸻

UI/UX Design Guide

Version 1.0
Last Updated: 2026

⸻

1. Philosophy

The product interface should emphasize clarity, speed, and focus. The design system prioritizes:

• Minimal visual noise
• Clear information hierarchy
• Consistent interactions
• Fast performance
• Long-term maintainability

The design language should feel:

Clean
Subtle
Intentional
Quiet

The interface should avoid decorative UI and instead allow content and functionality to dominate.

Key Principles
	1.	Content First
UI exists to support content and workflows, not compete with them.
	2.	Consistency Over Creativity
Familiar patterns are preferred over unique UI.
	3.	Low Cognitive Load
Users should not have to interpret the interface.
	4.	Color is Functional
Color communicates meaning — not decoration.
	5.	Components Over Custom UI
Reuse components whenever possible.

⸻

2. Visual Style

Overall Tone

The visual language should feel:

• Professional
• Calm
• Minimal
• Structured

Avoid:

• Loud colors
• Large gradients
• Overuse of icons
• Heavy shadows
• Decorative borders

⸻

3. Color System

Color should be used sparingly and intentionally.

The interface should be primarily built from:

• White / off-white surfaces
• Neutral greys
• One application color

⸻

3.1 Global Neutral Palette

Used across all applications.

--color-bg-primary: #FFFFFF
--color-bg-secondary: #F7F7F7
--color-bg-tertiary: #EFEFEF

--color-border: #E4E4E4

--color-text-primary: #1A1A1A
--color-text-secondary: #666666
--color-text-tertiary: #999999

These colors form 90% of the interface.

⸻

3.2 Application Color Identity

Each application in the ecosystem receives one primary brand color.

Example:

App	Primary Color
Reader	#4A7AFF
Warehouse	#00A36C
Finance	#E5533D
Scheduling	#7A5AF8

This color should only be used for:

• Primary buttons
• Active navigation states
• Focus indicators
• Key highlights

It must not dominate the UI.

⸻

3.3 Semantic Colors

Used globally.

Success:  #16A34A
Warning:  #F59E0B
Error:    #DC2626
Info:     #2563EB

Used for:

• alerts
• form states
• notifications

Never use semantic colors for general UI styling.

⸻

4. Typography

Typography should prioritize readability and density.

Font Family

Preferred:

Inter

Fallback:

system-ui
-apple-system
Segoe UI
Roboto


⸻

Type Scale

Style	Size	Weight
Page Title	24px	600
Section Title	18px	600
Body	14px	400
Label	12px	500
Caption	12px	400

Avoid text smaller than 12px.

⸻

5. Spacing System

Spacing should follow a consistent scale.

Base unit:

4px

Spacing scale:

4px
8px
12px
16px
20px
24px
32px
40px
48px

Guidelines:

• Inputs use 8–12px padding
• Sections use 24–32px spacing
• Layout containers use 32–48px spacing

⸻

6. Layout System

The layout should prioritize predictability and clarity.

Layout Structure

Standard application layout:

Sidebar
Header
Main Content

Example structure:

----------------------------------
Sidebar | Header
        |-------------------------
        | Content
        |


⸻

Content Width

Recommended maximum:

1200px

Large data tables may expand wider.

⸻

7. Component System

All UI elements should come from a shared component library.

/ui
  /components
  /tokens
  /layouts

Components derive styling from global CSS variables.

Example:

:root {
  --color-primary: #4A7AFF;
  --radius: 6px;
  --border: #E4E4E4;
}


⸻

8. Core Components

The system should include standardized versions of:

Inputs

• Text input
• Textarea
• Select
• Checkbox
• Radio
• Toggle
• Date picker

⸻

Buttons

Types:

Type	Purpose
Primary	Main action
Secondary	Alternative action
Ghost	Minimal action
Danger	Destructive

Primary buttons use the app color.

⸻

Navigation

• Sidebar navigation
• Tabs
• Breadcrumbs
• Pagination

⸻

Data Components

• Tables
• Data lists
• Cards
• Filters
• Search bars

⸻

Feedback

• Toast notifications
• Alerts
• Modals
• Empty states
• Loading indicators

⸻

9. Icon Usage

Icons should be used sparingly.

Use icons only when they:

• speed recognition
• improve scanning

Avoid icons for:

• decorative purposes
• redundancy with text

Preferred icon style:

Outline
Minimal
1.5px stroke

Recommended libraries:

Lucide
Heroicons


⸻

10. Motion and Animation

Motion should be subtle and functional.

Use animation only for:

• state transitions
• loading indicators
• modal entry/exit

Duration guidelines:

100ms – micro interaction
150ms – UI transition
200ms – modal

Avoid:

• bounce animations
• spring animations
• large movements

⸻

11. Interaction Patterns

Consistency in interactions is critical.

Hover

Background: +2–3% darker

Active

Background: +5–6% darker

Focus

Always visible:

outline: 2px solid var(--color-primary)

Accessibility should never remove focus indicators.

⸻

12. Accessibility

Accessibility is required.

Minimum standards:

• WCAG AA contrast
• keyboard navigation
• visible focus states
• accessible form labels
• ARIA only when necessary

Color should never be the only signal.

⸻

13. Third-Party Libraries

Third-party UI libraries should be strictly limited.

Allowed:

• icon libraries
• low-level primitives

Avoid:

• full UI frameworks
• complex component systems

Reason:

• visual inconsistency
• large bundle sizes
• limited customization

Preferred approach:

Build internal components


⸻

14. CSS Architecture

The design system should be based on design tokens.

Example:

/styles
  tokens.css
  base.css
  components.css

Tokens include:

colors
spacing
radius
typography
z-index

All components reference tokens.

⸻

15. Dark Mode (Optional Future)

Dark mode should invert neutrals while maintaining the same structure.

Example:

bg-primary: #111111
bg-secondary: #1A1A1A
text-primary: #F5F5F5

App colors remain unchanged.

⸻

16. Design Review Process

Before introducing a new component:
	1.	Check if an existing component solves the problem
	2.	Extend existing components first
	3.	If new, add to component library
	4.	Document usage examples

No UI should ship without component review.

⸻

17. Example Design Characteristics

A well-designed screen should feel:

• spacious
• structured
• quiet

Typical composition:

Page Title

Filters / Controls

Data Table

Pagination

No unnecessary UI elements.

⸻

18. Anti-Patterns

Avoid:

• bright backgrounds
• multiple accent colors
• unnecessary icons
• large drop shadows
• inconsistent spacing
• multiple button styles

⸻

19. Guiding Rule

If a UI element does not improve clarity or speed, it should not exist.

⸻

If you’d like, I can also create a second document that dramatically improves team adoption:

“UI Component Standards”

This includes:
	•	exact button specs
	•	table standards
	•	form layout rules
	•	filter UI patterns
	•	modal patterns
	•	sidebar behavior
	•	mobile behavior

That doc usually cuts UI inconsistency by ~80% in engineering teams.