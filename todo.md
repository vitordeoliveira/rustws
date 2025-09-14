# TODO List

## High Priority

- [ ] Use the livereloading of the core for all other projects
  - Share core livereload functionality across all workspace projects
  - Ensure consistent development experience
  - Priority: High

## Medium Priority

- [ ] Clean core lambdas dto to use enums properly

  - Refactor lambda DTOs to use proper enum types instead of current implementation
  - Priority: Medium
  - Urgency: Medium
  - Context: Improve type safety and code clarity in lambda data transfer objects

- [ ] Remove majority of not necessary js in the html files

  - Clean up HTML files by removing unnecessary JavaScript code
  - Priority: Medium
  - Urgency: Medium
  - Context: Reduce bundle size and improve page performance

- [ ] Check and improve the compile_rust to WASM in lambdas mod.rs
  - Enhance the compilation process to support dependency files for lambdas
  - Priority: Medium
  - Urgency: Medium
  - Context: Allow lambdas to have custom dependencies beyond the current hardcoded serde setup

## Low Priority

- [ ] Sync cursor rules in the projects

  - Ensure all projects have consistent cursor rule configurations
  - Priority: Low
  - Urgency: Low
  - Context: Share rule files across workspace projects for consistency

- [ ] Fix toaster to use alpineajax in the lambda page, when compile change status + toaster
  - Currently using existing implementation
  - Will fix in the future
  - Priority: Low
