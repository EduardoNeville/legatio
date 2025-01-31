![LegatioLogo](./docs/media/ascii-eagle.png)

# **Legatio: Beginner's Guide to Your Collaborative AI Assistant**

Welcome to **Legatio**, the innovative terminal-based tool for integrating AI into your file management and text editing workflows. This guide will walk you through the setup process, core features, and tutorials for using Legatio with various AI APIs (**OpenAI**, **Anthropic**, and **Ollama**). Each section provides step-by-step instructions for complete beginners.

---

## **What Does Legatio Do?**

Legatio empowers you to:
1. Manage **projects** as centralized workspaces.
2. Create and chain **prompts** for dynamic AI collaboration.
3. Attach and organize **scrolls** (external files) to provide detailed AI context.
4. Experiment across **branches** to explore ideas without impacting your main work.
5. Work with different AI APIs (OpenAI, Anthropic, Ollama) that align with your needs and goals.

---

## **Supported APIs**

Legatio currently supports the following AI providers:
- **OpenAI**: Suitable for versatile and general-purpose AI tasks (e.g., ChatGPT).
- **Anthropic**: Known for its Claude AI, useful for ethical and safe AI applications.
- **Ollama**: Great for lightweight, offline, or specific AI use cases.

Each API comes with its own configuration requirements and workflow. Follow the tutorials below to integrate them.

---

## **Getting Started**

### **Step 1: Install Prerequisites**

Before running Legatio, ensure your system meets these requirements:
1. **Rust Installed**: Install Rust if you haven’t already:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **Clone the Repository**: Clone the Legatio repository:
   ```bash
   git clone <repository-url>
   cd <repository-name>
   ```

3. **Build the Application**:
   ```bash
   cargo build --release
   ```

---

### **Step 2: Select an API**

Legatio supports multiple APIs. Choose one to proceed:

| API       | Recommended Use Case                                   |
|-----------|-------------------------------------------------------|
| OpenAI    | General-purpose tasks and ChatGPT integration         |
| Anthropic | Advanced ethical AI (Claude)                         |
| Ollama    | Local lightweight AI for offline or specific tasks    |

---

## **Tutorial 1: Using OpenAI API**

### Step 1: Register and Obtain an API Key
1. Visit [OpenAI](https://platform.openai.com/signup) and create an account.
2. Once logged in, generate your API key from the dashboard.

### Step 2: Export the API Key
Run the following command to set the key as an environment variable:
```bash
export OPENAI_API_KEY=your_openai_api_key
```

### Step 3: Edit `config.toml`
To ensure you're using OpenAI with Legatio:
1. Locate your `config.toml` file (generated in `$HOME/.config/legatio/`):
   ```toml
   [llm]
   llm = "openai"
   model = "gpt-4"  # You could also use gpt-3.5-turbo
   ```
   
2. Save and restart Legatio:
   ```bash
   cargo run --release
   ```

### **Step 4: Interact with the AI**
1. Create or select a project and add your prompt to `legatio.md`:
   ```markdown
   # ASK MODEL BELOW
   How does renewable energy benefit the environment?
   ```
2. Press `a` in the terminal to send your query and wait for the AI's response.

![Template OpenAI Integration](#)
*Alt Text: OpenAI-generated response to a user query in a terminal.*

---

## **Tutorial 2: Using Anthropic's Claude API**

### Step 1: Register and Obtain an API Key
1. Visit [Anthropic](https://www.anthropic.com/) to join their program and access API keys. Make sure to obtain the `ANTHROPIC_API_KEY`.

### Step 2: Export the API Key
Run the following command to set the key in your environment:
```bash
export ANTHROPIC_API_KEY=your_anthropic_api_key
```

### Step 3: Edit `config.toml`
Update your configuration file for Claude:
```toml
[llm]
llm = "anthropic"
model = "claude-2" # Replace with an available Claude model
```

### Step 4: Use Legatio with Anthropic
1. Launch Legatio:
   ```bash
   cargo run --release
   ```
2. Add a prompt to the `legatio.md` file inside your project:
   ```markdown
   # ASK MODEL BELOW
   What ethical considerations arise in AI-powered decision-making?
   ```
3. Press `a` to send the prompt to Anthropic's Claude, and results will be appended to `legatio.md`.

![Template Anthropic Integration](#)
*Alt Text: Anthropic's Claude response with ethical considerations in a terminal.*

---

## **Tutorial 3: Using Ollama (Offline AI)**

Ollama is suited for local environments where lightweight or fine-tuned AI is preferred.

### Step 1: Install Ollama
1. **Install Ollama CLI** based on your platform. For Mac:
   ```bash
   brew install ollama
   ```
2. Customize your LLM model (Ollama typically offers offline models).

### Step 2: Configure `config.toml`
Set Legatio to use Ollama and specify your local model:
```toml
[llm]
llm = "ollama"
model = "llama-2" # Replace with your local model name
```

### Step 3: Run Legatio
Launch the application:
```bash
cargo run --release
```

### Step 4: Add a Prompt for Ollama
1. Open your project's `legatio.md` file and input:
   ```markdown
   # ASK MODEL BELOW
   Provide a brief explanation of machine learning algorithms.
   ```
2. Press `a` to send the question. Ollama will respond and save answers locally.

![Template Ollama Integration](#)
*Alt Text: Ollama local AI response field displayed inside a terminal workspace.*

---

## **Managing Scrolls and Branches**

### Scrolls
Scrolls are project-specific files that add contextual information.
- **Add Scrolls**: Use the `n` key to add required files.
- **Remove Scrolls**: Delete them with the `d` key.

---

## **Branching**

Use Legatio's branching system to:
- Experiment on side branches.
- Leave the primary branch unaffected.

To switch branches, press `b` and follow the on-screen prompts.

![Template Branching Workflow](#)
*Alt Text: A branching workflow in Legatio where a user can switch between "Experimentation" and "Main Branch".*

---

## **Cheat Sheet**

| Key Combination | Action                                      |
|------------------|--------------------------------------------|
| `s`              | Select a project/prompts                  |
| `n`              | Create new project/scroll                 |
| `d`              | Delete project/prompt/scroll              |
| `e`              | Edit scrolls                              |
| `a`              | Interact with AI through the chosen API   |
| `b`              | Switch project branches                   |
| `p`              | Change the project                        |
| `q`              | Quit application                          |

---

## **Troubleshooting**

| Issue                        | Resolution                                                                 |
|------------------------------|---------------------------------------------------------------------------|
| Cannot connect to the AI     | Ensure API keys are set (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY`).           |
| API Errors                   | Double-check your `config.toml` is properly configured.                  |
| Failed API Calls in Ollama   | Verify Ollama CLI is installed and the selected model is loaded properly. |

---

## **Feedback**

Your feedback helps improve Legatio! If you find issues, have suggestions, or want to contribute, feel free to open a ticket or make a pull request on the GitHub repository.

---
