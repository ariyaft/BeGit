---
name: begit-ui-principles
description: Core design principles, UX patterns, and UI behaviors required for the BeGit application, curated from user preferences.
---

# BeGit UI/UX Design Principles

When contributing to or modifying the BeGit application, you MUST follow these specific principles and aesthetic guidelines as established by the project owner. The goal is a clean, modern, intuitive, and highly readable Git GUI.

## 1. Sidebar & Navigation Behaviors
- **Activity Bar over Complete Collapse**: The main left sidebar should never disappear completely. When collapsed, it must leave a persistent, narrow vertical "Activity Bar" (similar to VS Code) containing toggle icons (e.g., Source Control, Timeline).
- **Hover-based Interactions**: Expandable sections inside the sidebar (LOCAL, REMOTE, TAGS, etc.) should expand smoothly on mouse hover (`mouseenter`) and collapse when the mouse leaves (`mouseleave`), reducing the need for explicit clicks.

## 2. Graph & Timeline Visualizations
- **Horizontal Timeline Aesthetics (Gitflow Style)**: 
  - Commits should be represented by large, colorful circular nodes (e.g., `r=20`).
  - Branches/Lanes must be clearly labeled on the far left with colorful, rounded rectangular chips.
  - Parent/child relationships should be connected by clean grey paths with clear arrow markers pointing from older to newer commits.
  - Backgrounds should be clean (e.g., solid white) with subtle watermarks (like "GITFLOW").
- **Vertical Graph Aesthetics**:
  - Provide comfortable breathing room. Rows should be comfortably tall (e.g., `36px` row height).
  - The current `HEAD` commit must be immediately identifiable using a highly recognizable icon (like a solid yellow/gold star ⭐), both in the graph node and on the corresponding branch chip.

## 3. Branch Chips & Labels
- **No Annoying Hover Shifts**: Branch chips/labels should remain static in their layout flow when hovered. Do not use absolute positioning pop-outs that cause elements to shift visually.
- **Padding and Breathing Room**: Always ensure chips do not stick tightly to the edge of the sidebar or container (maintain at least `12px` of padding).
- **Merged Local & Remote Tracking**: If a local branch and its remote tracking counterpart (e.g., `main` and `origin/main`) point to the exact same commit, they MUST be merged into a single UI chip. This chip must display BOTH the local (monitor) and remote (cloud) icons side-by-side to clearly indicate its dual status, utilizing a dual-color border if supported.

## 4. Layout & Scrolling Management
- **Fluid Content Columns**: In list views (like the commit log), columns containing variable-length text (like commit messages) MUST be fluid. They should truncate long text with an ellipsis (`text-overflow: ellipsis`) rather than expanding indefinitely.
- **Preventing Empty Abyss**: Do not use `min-width: fit-content` on infinite-width containers if it causes massive empty horizontal scroll space after the columns. Calculate `min-width` dynamically based on the exact sum of the fixed columns, allowing the flexible columns to fill the remainder without stretching the viewport unnecessarily.

## 5. Feedback & Confirmation
- **Non-blocking feedback**: Use the shared toast system for errors, warnings, status updates, and unavailable actions. Do not use browser `alert()` dialogs anywhere in the application.
- **Immediate acknowledgement**: Start a visible, non-blocking activity indicator as soon as a Git operation is accepted, and keep it visible through the resulting refresh. Pair completion or failure with a toast so users can distinguish a delayed operation from an ignored click.
- **Required for new functionality**: When adding any asynchronous, user-triggered feature, include this feedback lifecycle from the outset: acknowledge immediately with the shared activity indicator, await the data refresh that changes the visible state, then clear the indicator and show a success or error toast. Do not leave an action without visible progress just because the operation eventually updates the UI.
- **Command audit trail**: Route every user-triggered Git mutation through the shared activity-log executor. Record the readable command, repository, timestamp, and success or failure; do not log passive background reads, which would make the activity history noisy.
- **Actionable messages**: State what failed or is unavailable and, when useful, the next action the user can take. Errors should stay visible long enough to read and remain dismissible.
- **Confirm intent, not routine feedback**: Use a confirmation control only before destructive or irreversible actions, such as aborting a Git operation, skipping conflict-resolution work, or deleting a branch. Do not use confirmations for ordinary success or failure messages.
