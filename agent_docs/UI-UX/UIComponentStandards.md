# UI Component Standards

Version 1.0
Last Updated: 2026

⸻

1. Purpose

This document defines the standard components and interaction patterns used across all applications.

Goals:

• Consistent user experience across products
• Faster UI development
• Reduced UI design decisions during implementation
• Reusable component architecture
• Predictable user behavior

All UI must be built using the shared component system.

⸻

2. Component Design Principles

All components should follow these principles.

2.1 Single Responsibility

Each component should do one thing well.

Examples:

✔ Button handles click interaction
✔ Table displays tabular data
✔ Modal displays blocking UI

Avoid components that mix responsibilities.

⸻

2.2 Composable

Components should support composition instead of customization.

Example:

Good

<Card>
  <CardHeader/>
  <CardContent/>
</Card>

Avoid overly configurable components with dozens of props.

⸻

2.3 Consistent States

All components must support consistent interaction states:

State	Required
Default	Yes
Hover	Yes
Active	Yes
Focus	Yes
Disabled	Yes


⸻

2.4 Predictable Layout

Components must maintain consistent spacing and sizing.

Spacing rules come from the global spacing scale.

⸻

3. Buttons

Buttons represent actions initiated by the user.

⸻

3.1 Button Types

Type	Usage
Primary	Main action on page
Secondary	Alternative actions
Ghost	Minimal UI actions
Danger	Destructive operations


⸻

3.2 Primary Button

Used for the single most important action.

Rules:

• Only one primary button per section
• Uses application accent color

Example usage:

Save
Create Order
Submit


⸻

3.3 Secondary Button

Used for supporting actions.

Examples:

Cancel
Back
Edit

Appearance:

• neutral background
• subtle border

⸻

3.4 Ghost Button

Minimal emphasis.

Used for:

• table actions
• toolbar actions
• inline actions

Example:

Edit
View
Duplicate


⸻

3.5 Danger Button

Used for destructive actions.

Examples:

Delete
Remove
Archive

Must always require confirmation.

⸻

3.6 Button Size Standards

Size	Height
Small	28px
Default	36px
Large	44px

Padding:

8px 16px


⸻

4. Form Inputs

Forms should prioritize clarity and efficiency.

⸻

4.1 Input Fields

Standard fields:

• Text Input
• Number Input
• Select
• Textarea
• Date Picker
• Checkbox
• Radio
• Toggle

⸻

4.2 Input Layout

Preferred form layout:

Label
Input
Help text (optional)
Error text (if present)

Example:

Email Address
[ input field ]

We will send receipts to this email.


⸻

4.3 Label Rules

Labels must:

• Always be visible
• Never rely on placeholders

Avoid:

Input with placeholder only


⸻

4.4 Input Width

Inputs should default to full container width.

Exceptions:

Field	Width
ZIP code	small
Quantity	small
Date	medium


⸻

4.5 Validation

Errors should appear directly below the input.

Example:

Email Address
[input]

Invalid email format

Error colors should use semantic error color only.

⸻

5. Tables

Tables are the primary way to display structured data.

⸻

5.1 Table Layout

Standard structure:

Table
  Header
  Rows
  Pagination

Optional:

Filters
Search
Bulk actions


⸻

5.2 Table Row Behavior

Row interactions:

Interaction	Behavior
Hover	Subtle background change
Click	Navigate to record
Select	Checkbox selection


⸻

5.3 Row Density

Standard row height:

44px

Compact mode:

36px


⸻

5.4 Table Actions

Row actions should appear as:

• trailing action buttons
• dropdown menu

Example:

Edit
Duplicate
Archive
Delete

Avoid cluttering rows with too many buttons.

⸻

6. Filters

Filters allow users to reduce dataset size.

Preferred placement:

Page title
Filters
Search
Table


⸻

6.1 Filter Types

Allowed filter styles:

• Dropdown filters
• Multi-select filters
• Date filters
• Search

⸻

6.2 Filter Behavior

Filters should:

• update results immediately OR
• apply via Apply button

Avoid both simultaneously.

⸻

7. Navigation

Navigation must remain consistent across applications.

⸻

7.1 Sidebar Navigation

Standard layout:

Logo
Primary Navigation
Secondary Navigation
Account

Example:

Dashboard
Orders
Products
Customers

Settings


⸻

7.2 Active State

Active navigation items must be visually distinct using:

• accent color
• background highlight

⸻

7.3 Collapsible Sidebar

Sidebar may support collapse:

Expanded

[icon] Orders

Collapsed

[icon]

Icons must remain clear.

⸻

8. Modals

Modals should be used sparingly.

Good uses:

• confirmation dialogs
• quick edits
• small forms

Avoid:

• large workflows

⸻

8.1 Modal Structure

Header
Body
Footer

Example:

Delete Order

Are you sure you want to delete this order?

Cancel   Delete


⸻

8.2 Modal Widths

Size	Width
Small	400px
Medium	600px
Large	800px


⸻

9. Notifications

Notifications provide system feedback.

⸻

9.1 Toast Notifications

Used for:

• success messages
• background actions

Example:

Order saved successfully

Duration:

3–5 seconds


⸻

9.2 Alerts

Alerts appear within pages.

Used for:

• warnings
• errors
• system notices

Example:

Your billing information needs updating.


⸻

10. Empty States

Empty states guide users when data is missing.

Structure:

Icon
Message
Action button

Example:

No Orders Yet

Create your first order.

[Create Order]


⸻

11. Loading States

Loading states should prevent layout shifts.

Preferred methods:

• skeleton loaders
• subtle spinners

Avoid:

• blank screens

⸻

12. Cards

Cards group related content.

Standard structure:

Card
  Header
  Content
  Footer (optional)

Cards should have:

• subtle border
• minimal shadow or none

⸻

13. Search

Search should appear in areas with large datasets.

Rules:

• always visible
• immediate response
• clear results

Placeholder example:

Search orders...


⸻

14. Pagination

Used when datasets exceed 25–50 items.

Standard placement:

Bottom right of table

Controls:

Previous
Page numbers
Next


⸻

15. Confirmation Dialogs

Required for destructive actions.

Example:

Delete Product

This action cannot be undone.

Cancel   Delete

Danger button must be clearly styled.

⸻

16. Mobile Behavior

Mobile interfaces should prioritize:

• vertical stacking
• simplified navigation
• touch-friendly targets

Minimum touch size:

44px

Tables should convert to:

stacked rows
or
cards


⸻

17. Component Documentation

Every component should include:

• usage description
• props / configuration
• examples
• accessibility notes

Stored in:

/ui/components/docs


⸻

18. Change Management

New components require:
	1.	Design review
	2.	Engineering review
	3.	Documentation update
	4.	Addition to component library

Avoid adding components for single use cases.

⸻

19. Final Rule

If a UI pattern already exists, reuse it.

Consistency is more important than optimization of individual screens.
