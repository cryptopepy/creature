# CREATURE - BasedAI - [Local Only, No Brains] - v0.2.1
> This project is a fork of **CREATURE - BasedAI** (the <a href="https://github.com/getbasedai/creature">original repostitory</a>). The *[What's new?](#whats-new)* section in the **Table of Contents** describes what has been added, updated and fixed. The *[Prerequisites](#prerequisites)* and *[Installation](#installation)* sections are also updated accordingly.
<img src="https://pbs.twimg.com/media/GcTMdVUXQAAmr24?format=jpg&name=4096x4096" alt="Image Description" width="500"/>
A self-organizing framework that combines cellular automata, coherence, and language models to explore emergent collective intelligence through cells that think, plan, and evolve in a multi-dimensional space. In the BasedAI version, this joins a collective and would work for a specific Brain.

## Table of Contents

- [What's new?](#whats-new)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Running the Creation](#creation)
- [Configuration](#configuration)
- [System Architecture](#system-architecture)
  - [Thought DNA Dimensions](#thought-dna-dimensions)
  - [Modules Overview](#modules-overview)
  - [Data Storage](#data-storage)
  - [Memory Management](#memory-management)
- [OpenRouter Integration](#openrouter-integration)
- [Monitoring and Visualization](#monitoring-and-visualization)
- [Data Analysis](#data-analysis)
- [Contributing](#contributing)
- [License](#license)

## What's new?

### Features

1. **OpenAI support**

  Latest OpenAI LLMs, like *o1-preview* and *GPT-4o*, are accessible as creature's analytic backend with [OpenAI API key](https://help.openai.com/en/articles/4936850-where-do-i-find-my-openai-api-key). Note that for using *o1* family of LLMs, you need to be OpenAI [Tier5](https://platform.openai.com/docs/guides/rate-limits/usage-tiers?context=tier-one#usage-tiers) customer.

2. **HuggingFace support**

  All text inference and text generation open-source models available in HuggingFace Inference are accessible as creature's analytic backend, both locally (via [lmdeploy](https://github.com/InternLM/lmdeploy)) and remotely (with [HuggingFace API key](https://www.geeksforgeeks.org/how-to-access-huggingface-api-key/)).

3. **Visualization page example**

<div align=center>
<img src=./assets/img/llama_3.2_initial_state.jpg width="80%"/>
</div>
  The colony currently running by your basedAI creature exposes various data about its state. This is an example page of how to display and use colony state data snapshot. It exposes the model, its token setting and framework currently being used by the colony, as well as nice 3D visualization of the colony's network of cells (alongside with hover tooltips with current cellular state details) and a matrix-like console with flowing cell details.

### Technical

## Prerequisites

1. **Install Rust and Cargo**

   Rust is the primary language used in this project. Install Rust and its package manager Cargo:

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

   For more detailed instructions, refer to the [official Rust installation guide](https://www.rust-lang.org/tools/install).

2. **Obtain an OpenRouter API Key**

   The system relies on the OpenRouter API for language model interactions such as thought generation, plan creation, memory compression, and context analysis. Sign up and obtain an API key from [OpenRouter](https://openrouter.ai/) and you can use crypto.

3. **Set Environment Variables**

   Set your API key in the environment variable:

   ```bash
   export OPENROUTER_API_KEY='your_openrouter_api_key_here'
   ```

   **Optional:** If you plan to use the Google Cloud Gemini model for advanced AI capabilities, set up your Google Cloud project and set the following environment variable:

   ```bash
   export GOOGLE_CLOUD_PROJECT='your_google_cloud_project_id'
   ```

   Ensure you have enabled the necessary APIs in your Google Cloud project and have proper authentication set up. Refer to the [Google Cloud documentation](https://cloud.google.com/docs/authentication) for guidance.

## Installation

1. **Clone the Repository**

   ```bash
   # Clone the repository
   git clone https://github.com/yourusername/creature.git
   cd creature
   ```

2. **Build the Project**

   ```bash
   # Build the project in release mode
   cargo build --release
   ```

   This compiles the project with optimizations, which is recommended for performance.

## Creation 

Run the simulation with default settings:

```bash
cargo run --release
```

### Customization

You can customize the simulation by providing a name and a mission statement:

```bash
cargo run --release -- --name "Dobby" --mission "Your mission statement here"
```

**Example:**

```bash
cargo run --release -- --name "Frogger" --mission "Explore emergent behaviors in decentralized systems"
```

### Command-line Options

- `--name`: Specify the name of your simulation or colony.
- `--mission`: Define the mission or goal guiding the simulation's behavior.
- `--api-key`: Provide the OpenRouter API key (alternatively can be set via OPENROUTER_API_KEY environment variable).
- `--batch-size`: Set the number of cells to process in each batch (default: 5).
- `--cycle-delay`: Set the delay between simulation cycles in milliseconds (default: 10ms).
- `--max-memory`: Set the maximum memory size per cell in bytes (default: 50,000 bytes).

## Configuration

Key configuration parameters are located in `models/constants.rs`. You can adjust these to modify the simulation's behavior:

- `MAX_MEMORY_SIZE`: Maximum memory size per cell (default: `50_000` bytes).
- `MAX_THOUGHTS_FOR_PLAN`: Maximum number of thoughts to consider when creating a plan (default: `42`).
- `BATCH_SIZE`: Number of cells processed in each batch (default: `5`).
- `CYCLE_DELAY_MS`: Delay between simulation cycles in milliseconds (default: `10` ms).
- `API_TIMEOUT_SECS`: Timeout for API calls in seconds (default: `300` seconds).

To change a constant, edit the value in `models/constants.rs` and rebuild the project.

## System Architecture

### Thought DNA Dimensions

The system operates across six key dimensions, forming each cell's "Thought DNA". The premise that the cells attempt to reach a steady-state in the ideas they form balancing the six.:

1. **Emergence** `(-100 to 100)`
   - Measures the development of novel properties and behaviors.
   - Influenced by thought generation and plan execution.

2. **Coherence** `(-100 to 100)`
   - Assesses system stability and coordination.
   - Affected by cell interactions and the success of plans.

3. **Resilience** `(-100 to 100)`
   - Evaluates adaptability to changes and recovery capabilities.
   - Enhanced through responding to challenges.

4. **Intelligence** `(-100 to 100)`
   - Gauges learning ability and decision-making quality.
   - Develops through thought evolution and experience.

5. **Efficiency** `(-100 to 100)`
   - Measures optimal resource utilization.
   - Improves with process optimization and waste reduction.

6. **Integration** `(-100 to 100)`
   - Reflects system connectivity and collaboration.
   - Strengthened by effective communication and teamwork among cells.

### Modules Overview

The project is organized into several key modules:

- **`api`**: Handles interactions with external APIs.
  - `gemini.rs` *(Optional)*: Implements `GeminiClient` for interacting with Google Cloud's Gemini AI Model.
  - `openrouter.rs`: Defines `OpenRouterClient` for making API calls to OpenRouter.
  - `mod.rs`: Exposes API clients for use in other modules.

- **`models`**: Contains data structures and constants.
  - `constants.rs`: Defines global constants for configuration.
  - `knowledge.rs`: Manages the knowledge base loaded from files.
  - `plan_analysis.rs`: Provides functionalities to analyze and save plan data.
  - `thought_io.rs`: Defines structures for event inputs and outputs.
  - `types.rs`: Defines core types like `CellContext`, `Thought`, `Plan`, and statistical data structures.
  - `mod.rs`: Exports commonly used types and structures.

- **`systems`**: Implements the core simulation logic.
  - `cell.rs`: Defines the `Cell` struct and its behaviors.
  - `colony.rs`: Manages the colony of cells and oversees simulation cycles.
  - `ltl.rs`: Implements logic for interaction effects and local temporal logic rules.
  - `ndarray_serde.rs`: Provides serialization for multi-dimensional arrays.
  - `basednodenet.rs`: Provides p2p communication between Brains. 
  - `mod.rs`: Exports key system components.

- **`interface`**: Provides a terminal-based user interface.
  - `widgets.rs`: Defines custom UI widgets like `CellDisplay` and `EnergyBar`.
  - `mod.rs`: Manages UI rendering and user interactions.

- **`server.rs`**: Sets up a WebSocket server for real-time monitoring. 

- **`utils`**: Contains utility functions.
  - `logging.rs`: Provides structured and colored logging utilities.
  - `mod.rs`: Exports utility functions.

- **`main.rs`**: The entry point of the application, orchestrating the simulation.

### Data Storage

The system maintains several data stores:

1. **`eca_state.json`**
   - Stores the full colony state, including all cells and their properties.
   - Used for persistence and potential recovery.
   - Updated each simulation cycle.

2. **`data/thoughts/`**
   - Contains individual thought records in JSON format.
   - Each file represents a thought with associated metadata like relevance scores and timestamps.

3. **`data/plans/`**
   - Stores executed and current plans.
   - Organized by cycle and plan ID.
   - Includes success metrics and analysis results.

### Memory Management

- **Thought Compression**
  - Cells have a memory limit defined by `MAX_MEMORY_SIZE`.
  - When the limit is reached, older thoughts are compressed to conserve memory.
  - Compressed memories retain essential information for future decision-making.

- **Historical Context**
  - The system leverages historical data to influence cell evolution.
  - Past experiences shape behavior, promoting adaptation over time.

## OpenRouter Integration

The system requires an OpenRouter API key for advanced language model capabilities:

- **Thought Generation**: Cells generate new thoughts based on their context and real-time data.
- **Plan Creation**: Cells synthesize thoughts into actionable plans.
- **Memory Compression**: Older memories are compressed to essential information.
- **Context Analysis**: Real-time context is gathered and analyzed to inform cell behaviors.

### Setting Up OpenRouter API Key

Ensure your API key is correctly set in your environment:

```bash
export OPENROUTER_API_KEY='your_openrouter_api_key_here'
```

The application reads this environment variable to authenticate API requests.

### Google Cloud Integration *(Optional)*

If you wish to use Google Cloud's Gemini model for additional AI capabilities, set the following environment variable:

```bash
export GOOGLE_CLOUD_PROJECT='your_google_cloud_project_id'
```

Refer to the [Google Cloud setup guide](https://cloud.google.com/docs/get-started) to configure your project and enable the necessary APIs.

**Note:** The Gemini integration is optional and requires additional setup, including authentication credentials and API enabling.

## Monitoring and Visualization

The system provides several ways to monitor and visualize the simulation:

- **Terminal User Interface (TUI)**
  - Real-time visualization within the terminal.
  - Displays cell grids, system logs, and status information.
  - Utilizes the `crossterm` and `ratatui` libraries for rendering.

- **WebSocket Server**
  - Runs on `localhost` port `3030`.
  - Allows external clients to connect and receive real-time updates.
  - Enables integration with custom dashboards or monitoring tools.

- **Logging**
  - Detailed, structured logs are output to the console.
  - Includes timestamps, metrics, and color-coded messages.
  - Uses custom logging utilities for enhanced readability.

## Data Analysis

Post-simulation analysis can be conducted by examining the data stored by the system:

- **View Latest Colony State**

  ```bash
  cat eca_state.json | jq
  ```

  Using `jq` helps format the JSON for readability.

- **Inspect Recent Thoughts**

  ```bash
  ls -l data/thoughts/
  ```

  Individual thought files can be viewed and analyzed for content and metadata.

- **Review Executed Plans**

  ```bash
  ls -l data/plans/
  ```

  Plans contain detailed steps and success metrics, which can be analyzed to understand the decision-making process.

- **Analyze Plan Performance**

  Since plans and their analyses are stored in JSON format, you can use tools like Python scripts, Jupyter notebooks, or data visualization software to parse and visualize the data.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

