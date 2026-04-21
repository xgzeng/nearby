# Comparison: Inquire vs. Ratatui vs. Cursive vs. Dialoguer

**Context:** Choosing a Rust TUI/CLI library for a configuration setup.
**Recommendation:** **Inquire** for simple linear setups; **Cursive** for form-heavy configurations.

## Quick Comparison

| Criterion | Inquire | Ratatui | Cursive | Dialoguer |
|-----------|---------|---------|---------|-----------|
| **UI Type** | Linear Prompt | Full Dashboard | Retained View | Basic Prompt |
| **Ease of Use**| Very High | Low (High Effort) | Medium | High |
| **Input Types**| Date, Password, MultiSelect | Custom | Dialog, TextArea, Menu | Basic Text, Confirm |
| **Boilerplate**| Near Zero | Significant | Moderate | Minimal |
| **Aesthetics** | Modern/Polished | High/Custom | Classic/Structured | Minimal/Basic |

## Detailed Analysis

### Inquire
**Strengths:**
- **Feature Rich:** Built-in date selection, password masking, and fuzzy-search selection out of the box.
- **Developer Experience:** Synchronous "call and return" API makes integration into existing code effortless.
- **Validation:** First-class support for closures that validate input as the user types.

**Weaknesses:**
- **Linear only:** Not suitable for non-linear forms (jumping from field 3 back to field 1).
- **Blocking:** Synchronous nature can block async runtimes if not handled correctly.

**Best for:** CLI wizards like `git init` or `npm init` style setup flows.

### Ratatui
**Strengths:**
- **Maximum Control:** Render anything at any pixel position.
- **Performance:** Highly optimized for real-time data visualization.
- **Ecosystem:** Huge library of community widgets (tui-textarea, tui-input).

**Weaknesses:**
- **Complex Logic:** You must manually track which field has focus, handle every keystroke (including Tab, Enter, Backspace), and manage the UI state machine.

**Best for:** System monitors, dashboards, or highly-branded terminal applications.

### Cursive
**Strengths:**
- **Automatic Layout:** Handles window positioning and view layering automatically.
- **Focus Management:** Tab/Shift-Tab focus switching is built-in.
- **Ease of Transition:** Feels similar to building a GUI (Qt/GTK).

**Weaknesses:**
- **Look and Feel:** Defaults to a more "classic" windowed aesthetic that may feel dated compared to modern flat-styled Ratatui apps.

**Best for:** Settings menus, file browsers, and multi-field forms.

### Dialoguer
**Strengths:**
- **Lightweight:** Very small dependency footprint.
- **Stable:** Highly tested and widely used in the ecosystem for years.

**Weaknesses:**
- **Feature Gap:** Lacks many of the modern input types (like Date) and advanced validation found in Inquire.

**Best for:** Minimalist tools that only need simple "Yes/No" or "Enter Name" prompts.

## Recommendation

**Choose Inquire when:**
- Your configuration setup is a linear sequence of 3-10 questions.
- You want the fastest possible development time with high-quality validation.

**Choose Cursive when:**
- You need a persistent "Settings" menu where users can jump between many fields.
- You want a robust, battle-tested UI without writing a custom focus management system.

**Choose Ratatui when:**
- You are building a flagship TUI product where the config is just one part of a larger, custom-rendered dashboard.

## Sources

- [Ratatui Official Site](https://ratatui.rs)
- [Inquire GitHub](https://github.com/mikaelmello/inquire)
- [Cursive GitHub](https://github.com/gyscos/cursive)
- [Dialoguer GitHub](https://github.com/console-rs/dialoguer)
