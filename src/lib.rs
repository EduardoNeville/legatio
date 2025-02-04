/// # Legatio: An AI-Powered Workflow Management Tool
///
/// `Legatio` is a terminal-based framework for managing and interacting with AI workflows. It allows users
/// to organize and structure projects, prompts, and text-based assets (scrolls) while providing mechanisms 
/// to interact with AI models for responding to questions or deriving insights.
///
/// This crate is specifically designed for developers, data scientists, or any other professionals who:
/// - Frequently use AI models (such as OpenAI's GPT) in their workflow.
/// - Require effective organization of data, logic, and context for these interactions.
/// - Want a CLI-based interactive UI for managing such workflows.
///
/// ## Features
///
/// - **Project Management:** Create and manage multiple projects, each having its own set of prompts and scrolls.
/// - **Prompt Chaining:** Chain prompts together as part of a Q&A interaction with AI models.
/// - **Scroll Management:** Organize and view text content from files (known as scrolls).
/// - **AI Interaction:** Query AI models directly from the CLI—supports configurable AI frameworks.
/// - **Customizable UI Themes:** Configure the visual appearance of the terminal UI.
/// - **Rich Terminal UI:** Uses tools like `ratatui` and `crossterm` to render a beautiful interactive interface.
///
/// ## Installation
///
/// Add the following dependency in your `Cargo.toml`:
/// ```toml
/// [dependencies]
/// legatio = "0.1.0"
/// ```
///
/// ## Getting Started
///
/// Here is how you can use `Legatio` in your project:
///
/// ### Step 1: Initialize the application
///
/// ```rust
/// use legatio::Legatio;
/// use sqlx::SqlitePool;
/// use anyhow::Result;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     // Create a connection pool to SQLite
///     let pool = SqlitePool::connect("legatio.db").await?;
///
///     // Instantiate and run the Legatio application
///     let mut app = Legatio::new();
///     app.run(&pool).await?;
///
///     Ok(())
/// }
/// ```
///
/// ### Step 2: Navigating in the Terminal UI
///
/// Once the application launches, use the following keyboard shortcuts to navigate between different operations:
///
/// - **Project Selection:**
///   - `[s]`: Select a project.
///   - `[n]`: Create a new project by selecting a folder.
///   - `[d]`: Delete a selected project.
///   - `[q]`: Quit the application.
///
/// - **Prompt Management:**
///   - `[s]`: Select a prompt for asking the AI model.
///   - `[d]`: Delete a selected prompt.
///   - `[e]`: Edit associated scrolls.
///   - `[p]`: Go back to project selection.
///   - `[q]`: Quit the application.
///
/// - **Scroll Management:**
///   - `[n]`: Add a new scroll (select a file).
///   - `[d]`: Delete a selected scroll.
///   - `[a]`: Ask the AI model with the current scroll/prompt context.
///   - `[s]`: Switch back to prompt selection.
///   - `[q]`: Quit the application.
///
/// - **Asking AI Models:**
///   - `[a]`: Send the current prompt chain and context to the AI model.
///   - `[b]`: Go back to prompt selection.
///   - `[e]`: Edit associated scrolls.
///   - `[p]`: Change the current project.
///   - `[y]`: Confirm an AI query.
///   - `[n]`: Cancel an AI query.
///
/// ### Step 3: Project Structure
///
/// Each project created in `Legatio` is structured with the following components:
///
/// - **Scrolls:** Files and other text-based assets that act as contextual data for AI queries.
/// - **Prompts:** Instructions or questions used when interacting with the AI model.
/// - **Prompt Chains:** A series of prompts linked together that form a context-sensitive conversation.
///
/// For example, you might create a project for "Technical Documentation," with multiple scrolls like:
/// - `architecture_overview.txt`
/// - `design_decisions.md`
///
/// You might also add prompts like "Explain the architectural diagram" or "Summarize the design rationale."
///
/// ### Step 4: Customizing AI Parameters
///
/// `Legatio` allows fine-grained control over how you interact with AI models:
///
/// - Set the framework (e.g., OpenAI, custom LLMs).
/// - Specify the AI model (e.g., `gpt-4` or `chatgpt-4o-latest`).
/// - Configure the maximum tokens that the model should generate.
///
/// The configuration is managed automatically but can be edited manually by modifying the user configuration
/// stored on disk.
///
/// ## Example Workflow
///
/// Here's an example illustrating a typical workflow:
///
/// 1. Launch the application.
/// 2. Select an existing project or create a new project by selecting a folder.
/// 3. Load scrolls from relevant text files (e.g., `.md`, `.txt`, etc.).
/// 4. Add prompts for querying the AI model (e.g., "Explain this file's contents").
/// 5. Chain prompts together for more complex queries.
/// 6. Ask the AI model for output and store the response as part of the prompt chain.
/// 7. Repeat steps 4–6 for iterative refinement.
///
/// ## Advanced Usage
///
/// If you want to build tooling on top of `Legatio`, you can utilize the public API:
///
/// - Use `Legatio::new()` to create an instance of the application.
/// - Use various helper methods for managing projects (`store_project`, `delete_project`, etc.).
/// - Build custom workflows by directly interacting with AI models via `ask_ai` queries.
///
/// ## Dependencies
///
/// The crate relies on the following major libraries:
/// - **`ratatui`**: For terminal-based user interfaces.
/// - **`crossterm`**: For keyboard input and terminal rendering.
/// - **`sqlx`**: For SQLite database interactions.
/// - **`tokio`**: For asynchronous runtime.
/// - **`anyhow`**: For error handling.
/// - **`ask_ai`**: A library for interacting with AI models.
///
/// ## Contributing
///
/// Contributions, bug reports, and feature requests are welcome! 
/// Please open an issue or submit a pull request on the [GitHub repository](https://github.com/your-username/legatio).
///
/// ## License
///
/// `Legatio` is licensed under the MIT License. See the LICENSE file for details.
///
/// ---
///
/// With `Legatio`, you can create a streamlined, interactive, and AI-driven workflow for managing projects.
/// Whether you're leveraging AI for documentation, ideation, or code generation, Legatio empowers you to stay organized and productive.
pub mod core;
pub mod services;
pub mod utils;
