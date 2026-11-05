# CodeCraft

CodeCraft is a modern, lightweight desktop Integrated Development Environment (IDE) built as a custom alternative to IntelliJ IDEA. It is designed to provide a clean, responsive workspace for code editing, project navigation, and application output monitoring.

---

## Key Features

* **Project Explorer Sidebar:** A directory browser with file-type icon support and shortcuts for managing files and folders.
* **Multi-Tab Code Editor:** An editing workspace featuring line numbering, active line highlights, and syntax highlighting across multiple open files.
* **Integrated Execution Panel:** A collapsible bottom panel containing an interactive terminal, build output stream, and debug log.
* **Clean Room UI:** An IntelliJ-inspired dark interface optimized for clarity and reduced eye strain.
* **Header Controls:** Dedicated action buttons in the top toolbar to build, run, and debug projects.

---

## System Requirements and Tech Stack

* **Core Runtime:** Electron / Web Components
* **Primary Languages:** TypeScript / JavaScript (or Python backend)
* **Supported Platforms:** Linux, macOS, Windows

---

## Quick Start

### 1. Clone the Repository
```bash
git clone https://github.com/your-username/CodeCraft.git
cd CodeCraft
```

### 2. Install Dependencies
```bash
npm install
```

### 3. Launch the Application
```bash
npm start
```

---

## Repository Structure

```text
CodeCraft/
├── assets/            # Application icons and graphic assets
├── components/        # User interface modules
│   ├── Editor/        # Syntax highlighting and editor logic
│   ├── FileTree/      # Directory tree browser component
│   └── Terminal/      # Output console and terminal emulator
├── src/               # Application entry point and core logic
├── package.json       # Project dependencies and scripts
└── README.md          # Project documentation
```

---

## Contributing

Contributions, bug reports, and feature requests are welcome. Please submit an issue or pull request through the repository's issues page.

---

## License

This project is licensed under the MIT License. Refer to the `LICENSE` file for details.
