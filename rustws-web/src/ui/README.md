# UI Structure Standard

## Module Organization

Each UI module should follow this standard structure:

```
module_name/
├── mod.rs          # Module exports and public interface
├── ui.rs           # UI rendering logic and handlers
└── *.html          # HTML templates for the module
```

## Core UI Files

```
ui/
├── engine.rs       # Template engine configuration and setup
├── filters.rs      # Custom template filters and helpers
├── mod.rs          # Main UI module exports
├── testers.rs      # UI testing utilities
└── shared/         # Shared UI components and layouts
    ├── mod.rs
    ├── layouts.rs  # Common page layouts
    └── base.html   # Base template
```

## File Responsibilities

### `ui.rs`

- Template rendering functions
- UI state management
- Component composition
- Template context preparation

### `*.html`

- HTML templates using template engine syntax
- Component markup and structure
- Alpine.js directives for interactivity

### `mod.rs`

- Public exports from the UI module
- Module interface definition

### `engine.rs`

- Template engine initialization
- Global template configuration
- Template loading and caching

### `filters.rs`

- Custom template filters
- Helper functions for templates
- Data formatting utilities

## Naming Conventions

- Module names: snake_case (e.g., `user_profile`, `admin_panel`)
- Template files: snake_case.html (e.g., `login.html`, `user_settings.html`)
- Function names: snake_case (e.g., `render_dashboard`)

## When to Create a New UI Module

Create a new UI module when you have:

- A distinct page or feature section
- Separate templates and UI logic
- Independent styling or behavior
- Clear UI boundaries from other modules

## Template Organization

- Keep templates close to their rendering logic
- Use descriptive filenames that match their purpose
- Group related templates in the same module
- Use `shared/` for reusable components and layouts
