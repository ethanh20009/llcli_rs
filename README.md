# LLCLI_RS - A CLI Tool for using LLMs - Built with Rust

LLCLI_RS is a blazing-fast, robust, and versatile command-line interface (CLI) for interacting with Large Language Models (LLMs).
Built with the power and safety of Rust,
LLCLI_RS allows you to seamlessly integrate LLMs into your workflow,
whether you're crafting creative content, automating tedious tasks, or exploring the boundless possibilities of AI.

## Shell Autocomplete
![cli_autocomplete](https://github.com/user-attachments/assets/dc08e02d-21c8-4b69-936f-9be5c2f2bc3b)
> Credit to [fish-ai](https://github.com/Realiserad/fish-ai) for fish integration strategy.

## CLI
<https://github.com/user-attachments/assets/81b6b474-54d2-4dc2-80bc-e9f96b665e7a>

## Note

Currently, LLCLI_RS is very work-in-progress.
Expect lots of breaking changes during development.

## Key Features

- **Rust-Powered Performance:** Experience the speed and reliability of Rust, ensuring efficient and stable LLM interactions.
- **Versatile Integration (in progress)** Easily connect to various LLM providers and APIs, allowing you to choose the best model for your specific needs.
- **LLM-Tool-Support:** Use the power of modern LLMs tool based capabilities, such as searching the web, all configurable with the CLI.
- **Streamlined Workflow:** Quickly generate text, translate languages, summarize documents, and more, all from the comfort of your terminal.
- **Customizable & Extensible:** Configure prompts, parameters, and outputs to tailor the LLM's behavior to your exact specifications. Explore custom commands and potential plugin architecture for advanced functionality.
- **User-Friendly Interface:** Enjoy a clean and intuitive command-line experience that makes working with LLMs a breeze (with markdown rendering support).
- **Scripting-Support:** LLCLI_RS has been made with scripting in mind, giving you the power of using the raw text model responses when desired.
- **Shell-Integration:** Inspired by [fish-ai](https://github.com/Realiserad/fish-ai), LLCLI_RS supports AI-powered shell autocomplete.

## Install from source

### Requirements:
- Cargo [https://doc.rust-lang.org/cargo/getting-started/installation.html](https://doc.rust-lang.org/cargo/getting-started/installation.html)
#### (Linux)
- **Dbus-based Secret Service**: Use [Gnome-Keyring](https://wiki.gnome.org/Projects/GnomeKeyring)

### No shell integration
Simply clone and run `./scripts/install.sh`

### Fish
For `fish` terminal support, run `./scripts/install.sh --term fish`

## Supported Integrations (Work-In-Progress)

- [x] Gemini
- [ ] OpenAI
- [ ] Claude
- [ ] Ollama
- [ ] Deepseek
