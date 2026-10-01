# BeGit

BeGit is a modern, fast, and visually stunning Git graphical user interface (GUI) designed to make version control intuitive and beautiful. Built on top of **Tauri**, **Vue 3**, and **Rust**, BeGit offers lightning-fast performance combined with a premium, fluid user experience.

## ✨ Key Features

- **Beautiful Visualizations:**
  - **Vertical Commit Graph:** An interactive, GitKraken-style timeline of your commits. Features comfortable row spacing, merged local/remote branch tracking chips, and a highly visible gold star ⭐ for your active `HEAD`.
  - **Horizontal Timeline (Gitflow View):** A creative, horizontally scrolling visualization that beautifully maps out your branches in color-coded lanes, connecting parent/child commits with clean bezier curves and directional arrows.

- **Smart Navigation:**
  - **Activity Bar:** Just like VS Code, collapsing the main sidebar leaves a slim, persistent Activity Bar to let you quickly toggle between your Source Control view and Horizontal Timeline view.
  - **Hover-based Interactions:** Sidebar sections (Local Branches, Remote, Tags, Stashes) smoothly expand on hover, minimizing unnecessary clicks.
  - **Smart Layouts:** Fluid, dynamic columns that gracefully handle long commit messages with clean text truncation, never leaving you lost in massive empty scrolling abysses.

- **Powerful Git Operations:**
  - Push, pull, fetch, stash, and pop directly from the primary graph toolbar.
  - Right-click context menus for branches, remotes, stashes, and commits.
  - View rich file diffs directly inside the app with syntax highlighting.
  - All core operations are powered natively by a **Rust** backend using `git2` and raw command execution for maximum speed.

## 🛠️ Tech Stack

- **Frontend:** Vue 3 (Composition API, `<script setup>`), TypeScript, Vanilla CSS for maximum styling control.
- **Backend / Desktop Framework:** Tauri + Rust.
- **Icons:** Custom SVG icons.

## 🚀 Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri Dependencies](https://tauri.app/v1/guides/getting-started/prerequisites) (e.g., Xcode command line tools on macOS).

### Installation & Development

1. **Install NPM dependencies:**
   ```bash
   npm install
   ```

2. **Run the application in Development Mode:**
   ```bash
   npm run tauri dev
   ```
   This will spin up both the Vite development server and the Tauri Rust application with Hot Module Replacement (HMR) active.

### Building for Production

To compile the application into a standalone binary for your operating system:

```bash
npm run tauri build
```

## 🎨 UI/UX Design Principles

This project strongly enforces a specific set of design principles. If you are contributing, please review the AI Skill file located at `.agents/skills/begit-ui-principles/SKILL.md` to ensure your layouts, hover states, and graphical elements match the established BeGit aesthetic.
