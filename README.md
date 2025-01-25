# **Legatio: Collaborative AI Text Assistant**

Legatio is a feature-rich, extensible, and user-centric tool for collaborative text editing, project branching, and AI-powered assistance. Designed with a focus on productivity and seamless user interaction, Legatio integrates a minimalist UI, project and prompt management, and advanced AI models (like OpenAI's GPT) to help users create, manage, and enhance their projects effectively.

This documentation will provide you with a thorough walkthrough of what Legatio does, how to set it up, and how to use its key features, such as managing `.md` files (Markdown), branching projects, and working with files.

---

## **Features**

- **Project Management**: Organize your workflow using projects that represent isolated tasks, topics, or objectives.
- **Prompt Storage & Chaining**: Store and chain AI-generated prompts together in an orderly manner.
- **`.md` File Integration**: Use a `legatio.md` file for keeping track of your session, generating new content, and interacting with the AI.
- **Branching Mechanism**: Work on different "branches" within the project, enabling isolated pipelines of thought and experimentation.
- **Scroll Management**: Attach related files (referred to as "scrolls") to a project for context and enrich your AI interactions.
- **Interactive Terminal UI**: A clean, TUI (Terminal User Interface) experience built with `ratatui`.

---

## **Getting Started**

### **Prerequisites**
1. Rust installed on your machine. You can install Rust via Rustup by following the instructions [here](https://www.rust-lang.org/tools/install).
2. OpenAI API Key. Sign up [here](https://platform.openai.com/signup) if you don't already have an API Key.
3. An OpenAI-enabled account with permission to use GPT-based models.

### **Setup**
1. Clone this repository:
   ```
   git clone <repository-url>
   cd <repository-name>
   ```

2. Install dependencies:
   ```
   cargo build --release
   ```

3. Set your OpenAI API Key:
   ```
   export OPENAI_API_KEY=your_openai_api_key
   ```

4. Run the application:
   ```
   cargo run --release
   ```

---

## **How to Use**

### **Legatio Workflow**
1. **Projects as Workspaces**:
   - A project in Legatio represents a "workspace" where you can manage prompts, integrate scrolls (files), and interact with the AI.
   - Each project is tied to a directory in your file system.

2. **The `.md` File**:
   - Every project has a key accompanying file named `legatio.md`, where all AI-generated interactions, prompts, and outputs are logged and stored.
   - `legatio.md` serves as both a record and a workspace for interacting with the tool.

     **Example Structure of the File**:
     ```markdown
     # PROMPT 1
     Write a summary for the article on climate change.
     
     # OUTPUT 1
     Climate change refers to long-term shifts in temperatures and weather patterns, primarily caused by human activities...
     
     # ASK MODEL BELOW
     Analyze the impact of fossil fuel usage on global average temperatures.
     ```

   - The content below the `# ASK MODEL BELOW` marker in the file is treated as "new input" for the AI.

3. **Scrolls**:
   - Scrolls are external files attached to a project that contain useful context or reference material required for completing tasks.
   - These scrolls are read and processed by the AI while responding to prompts.
   - Scrolls are managed through the `EditScrolls` mode in the UI.

### **Navigating Legatio's Modes**

Legatio follows a structured pipeline wherein a user moves between different **states**. Below is an explanation of the key states and how to interact with them:

#### 1. **Project Management**

- **[s] Select Project**: Select an existing project or directory to work on.
- **[n] New Project**: Create a new project directly in the UI. You'll be asked to provide a directory.
- **[d] Delete Project**: Permanently remove a project from the system.
- **[q] Quit**: Exit the application.

#### 2. **Prompt Management**

- **[b] Select Prompt**: Choose an existing prompt within the project to continue working with the AI.
- **[d] Delete Prompt**: Remove a specific prompt from the project.
- **[e] Edit Scrolls**: Manage scrolls associated with the project.
- **[p] Change Project**: Switch to a different project.
- **[q] Quit**: Exit the application.

#### 3. **Asking the AI**

- **[a] Ask the Model**: Enter your query for the model to generate a response.
- **[b] Switch Branch**: Navigate or move between branches.
- **[e] Edit Scrolls**: Add, edit, or remove scrolls.
- **[p] Change Project**: Alternate between projects.
- **[q] Quit**: Exit.

#### 4. **Scroll Management**

- **[n] New Scroll**: Attach new files to the project directory as scrolls.
- **[d] Delete Scroll**: Remove an associated scroll from the project.
- **[a] Ask Model**: Access the AI interaction with the project's context.
- **[s] Switch Branch**: Return to the main branch of the project.
- **[p] Change Project**: Exit to a different project.
- **[q] Quit**: Exit the application.

---

### **Branching in Legatio**
Branching is a powerful feature of Legatio that allows you to:
1. Create isolated versions of a project where you try out different ideas without affecting the parent branch.
2. Experiment with specific chains of prompts while maintaining the main project structure.

#### How It Works
- Branching allows you to switch to a specific segment of your ongoing project.
- You can view, modify, and chain your prompts differently for each branch.

#### Real-World Example
- **Branch A**: Generates a concise executive summary.
- **Branch B**: Creates a long-form, in-depth analysis.

Switching branches dynamically pulls all relevant project context (prompts, scrolls, etc.) into the workspace.

---

### **AI Workflow**
Here's how prompts work with Legatio and the AI:
1. System Prompts:
   - Scrolls are processed into a **system prompt**, offering background information to the AI.

2. Input Prompts:
   - User-defined inputs (or prompts) are used to query the model.

3. Prompt Chaining:
   - Legatio supports chaining prompts together, allowing context from previous prompts to be fed into new queries.

4. Output Storage:
   - AI responses are stored alongside their associated prompts in the `legatio.md` file.

---

### **Shortcuts Table**
| Shortcut       | Action                          |
|----------------|---------------------------------|
| `s`            | Select Project                 |
| `n`            | Create New Project/Scroll      |
| `d`            | Delete Project/Prompt/Scroll   |
| `e`            | Edit Scrolls                  |
| `a`            | Ask AI Model                   |
| `b`            | Switch Branch                  |
| `p`            | Change Project                 |
| `q`            | Quit the Application           |

---

## **Best Practices**

1. **Divide and Conquer**:
   Use branching to divide large tasks into manageable sub-segments.

2. **Organize with Scrolls**:
   Attach relevant documentation, context files, or external research to projects for more effective AI responses.

3. **Utilize Chains**:
   When working on a project with multiple prompts, use chaining to maintain coherency in output.

4. **Maintain the `.md` File**:
   The `legatio.md` is your source of truth for all interactions in a specific project; review it periodically for clarity and structure.

