# Legatio

## Overview

**Legatio** is a robust project-driven terminal tool aimed at streamlining workflows for developers, content creators, and AI enthusiasts. It leverages SQLite as the database backend and integrates seamlessly with OpenAI's API for prompt-based model interactions. Legatio is designed to manage projects, prompts, and artifacts like "scrolls" within an organized environment, enabling efficient collaboration and high-quality outputs.

Whether you're brainstorming creative ideas, building a knowledge repository, or working on a software project, **Legatio** is designed to keep everything in one place while giving you complete control over interactions with language models and your filesystem.

---

## Features
- **Project Management**: Organize your work into specific projects, each containing editable prompts and scrolls.
- **Prompt Interaction**: Chain prompts, view histories, and engage with OpenAI models efficiently.
- **Scroll Integration**: Append or remove scrolls as resources for AI context.
- **Versioned Workflows**: Save, edit, and reuse model interactions to build a scalable history of your work.
- **Dynamic Interface**: Provides a terminal-based UI for navigation and seamless experience.
  
---

## Prerequisites
1. **Rust**: Ensure you have Rust installed. If not, download and install it from [Rust's official website](https://www.rust-lang.org/).
2. **SQLite**: No separate installation is required for SQLite; this project uses the `sqlx` crate.
3. **OpenAI API Key**: The project interacts with OpenAI's API. Obtain your API key from the [OpenAI Dashboard](https://platform.openai.com/signup).

---

## Installation

### 1. Clone the Repository
```bash
git clone https://github.com/your_username/legatio.git
cd legatio
```

### 2. Environment Configuration
Create a `.env` file in the project root directory and add the following environment variables:
```
# File: .env
OPENAI_API_KEY=your_openai_api_key
DATABASE_URL=sqlite://legatio.db
```

### 3. Build and Run
Build the project with Cargo:
```bash
cargo build --release
```
Run the application:
```bash
cargo run
```

---

## Usage Guide

### Starting the Application
After running `cargo run`, the application will initialize by checking for existing projects. If you don’t have a project, you'll be prompted to create a new one.

---

### **Navigation Workflow**

#### 1. **Main Menu**
Choose between creating a new project or selecting an existing project.

#### 2. **Project Operations**
Once a project is selected, you can:
- View saved prompts and select one to expand.
- Interact with the model using existing prompts.
- Add, edit, or remove scrolls associated with the project.

#### 3. **Prompt Interaction**
Within a project, you can:
- Chain existing prompts into structured discussions.
- Generate new outputs with OpenAI's API.
- Save outputs back into the system for future reference.

#### 4. **Scroll Management**
Attach or remove "scrolls" (artifacts such as files or references) which enhance the context of AI interactions.

#### 5. **Exit the Application**
To exit, choose the "Exit" option from any of the menus.

---

### Example Workflow

1. **Create a New Project**:
    - Choose `New Project` when prompted.
    - Select a directory from your filesystem.
    - A new project will be initialized, and you can start configuring it with prompts and scrolls.

2. **Add Scrolls**:
    - In the `Edit Scrolls` section, append new resources (e.g., text files) to your project for use in model prompts.

3. **Interact with OpenAI's Model**:
    - Use the `Ask the Model` feature to load scrolls, system prompts, and engage in meaningful LLM-based interactions.

4. **Save Outputs**:
    - Output from the model is appended to the project's `legatio.md` file and saved as a prompt chain.

---

## File Structure

Legatio organizes files systematically within your projects. Here is an example structure:
```
/my-project
    ├── legatio.db        # SQLite database used for storing projects, prompts, and scroll metadata.
    ├── project_1/
    │   ├── legatio.md    # AI interaction history and saved prompts for this project.
    │   ├── scroll_1.txt  # Attached scroll file.
    ├── project_2/
    │   ├── legatio.md
    │   ├── scroll_2.txt
```

---

## Development Notes

### Database
The project uses SQLite as the primary datastore. The `legatio.db` file will automatically generate at runtime.

### Logging
Legatio includes logging functionality for tracking issues during runtime. Logs are printed to the terminal when `env_logger::init()` is called.

---

## Videos and Screenshots (Placeholder)

Below are placeholders where you can later include screenshots or videos demonstrating the user workflow.

### Screenshots
- **Project Selection Screen**
  ![Project Selection Screen](screenshots/project_selection.png)

- **Prompt Interaction**
  ![Prompt Interaction](screenshots/prompt_interaction.png)

### Video Guide
- **Video Placeholder**  
[![Video Walkthrough](https://img.youtube.com/vi/sample_video_id/0.jpg)](https://www.youtube.com/watch?v=sample_video_id)

---

## Contribution
We welcome contributions! To contribute:
1. Fork the repository.
2. Create a new branch and make necessary changes.
3. Submit a pull request.

Make sure your code adheres to Rust best practices and passes existing tests.

---

## License
This project is licensed under the MIT License. See [LICENSE](LICENSE) for more details.

---

## Contact
For questions or feedback, reach out:
- **Email**: `your_email@example.com`
- **GitHub Issues**: [Report an Issue](https://github.com/your_username/legatio/issues)

---

Enjoy using **Legatio** to supercharge your AI-assisted workflows! 🚀
