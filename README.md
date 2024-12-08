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

  Latest OpenAI LLMs, like *o1-preview* and *GPT-4o*, are accessible as creature's analytic backend with [OpenAI API key](https://help.openai.com/en/articles/4936850-where-do-i-find-my-openai-api-key).  
  **Note:** For using *o1* family of LLMs, you need to be OpenAI [Tier5](https://platform.openai.com/docs/guides/rate-limits/usage-tiers?context=tier-one#usage-tiers) customer.

2. **HuggingFace support**

  All text inference and text generation open-source models available in HuggingFace Inference are accessible as creature's analytic backend, both locally (via [lmdeploy](https://github.com/InternLM/lmdeploy)) and remotely (with [HuggingFace API key](https://www.geeksforgeeks.org/how-to-access-huggingface-api-key/)).

3. **Replicate support**

  Text generation using models available in [Replicate](https://replicate.com/) is accessible as creature's analytic backend (with [Replicate API key]([https://www.geeksforgeeks.org/how-to-access-huggingface-api-key/](https://replicate.com/account/api-tokens))).

4. **Visualization page example**

<div align=center>
<img src=./assets/img/llama_3.2_initial_state.jpg width="80%"/>
</div>
  
  The colony currently running by your basedAI creature feeds various data about its state through a web socket. The above is a screenshot of a [three.js](https://threejs.org/)-based web page, which is present in this repository, showing how to use and visualize colony state data snapshots. It exposes the model, its token setting and framework, currently being used by the colony, as well as nice 3D visualization of the colony's network of cells (alongside with hover tooltips with current cellular state details) and a matrix-like console with flowing cell details.

### Technical

1. **Features**
   - HuggingFace, Replicate and OpenAI interfaces are copy of the OpenRouter interface, with adapted REST API calls and operational paramteres, expected from in the environment.
   - Abstract API interface (enum), incorporating all 3 types of interfaces - OpenRouter, HuggingFace and OpenAI.
   - Visualization page example, based on three.js 3D animations and GPT-4-derived The Matrix-like styling. Asynchronous websocket handling and visualization, keeping the colony cell states in a temporary storage.
   - Debugging LLM I/O and IPC tracking (for race conditions and deadlocks debugging).
   
2. **Fixes**
   - Demoted the colony *Mutex* to *RWLock*, to clamp the deadlock, plaguing the creature once its websocket is being connected.
   - Replaced *PlanAnalysis:analyze_plans()* best plan assessment *partial_cmp()* with *total_cmp()* to bypass malformed plans. **!!! Potential bug masking here !!!**
   - Text parsing thought components
   
3. **Workarounds**
   - Removed most of the data sending events from server.rs, such as initial event and the heartbeat, leaving only the snapshot. The reason behind this is inherent problem in the thought analysis architecture, which requires the whole colony object write-locked, which in turn locks the read access to it from the websocket.

4. **TODO**
   - Prompt partitioning and optimizations for different frameworks (that is - HuggingFace and OpenAI).
   - Fix colony's thought process prolonged RWLock hold by applying IPC on smaller code granularity.
   - Code organization and structure - remove the induced spaghettification and apply modularization (for example - of all prompts).

## Prerequisites

1. **Linux (Debian-clone - like Debian, Ubuntu, Kali, etc., including the ones installed under Windows WSL (Windows Sub-system Linux)) Installation**
   
   Update your system and install the required packages:
   ```bash
   sudo apt-get update && sudo apt-get upgrade -y && sudo apt-get install -y pkg-config libssl-dev build-essential libc++-dev libc++abi-dev cargo clang
   ```

3. **Install Rust and Cargo**

   Rust is the primary language used in this project. Install Rust and its package manager Cargo:

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

   For more detailed instructions, refer to the [official Rust installation guide](https://www.rust-lang.org/tools/install).

4. **Obtain an API Key**
   
   The system relies on API for language model interactions, such as thought generation, plan creation, memory compression, and context analysis.
   
   - **Using OpenRouter**

     Sign up and obtain an API key from [OpenRouter](https://openrouter.ai/) and you can use crypto.
     
   - **Using OpenAI**  

     Sign up and obtain an API key from [OpenAI](https://help.openai.com/en/articles/4936850-where-do-i-find-my-openai-api-key).
     
   - **Using HuggingFace**  

     Sign up and obtain an API key from [HuggingFace](https://www.geeksforgeeks.org/how-to-access-huggingface-api-key/). **You don't need an API key if you plan to run non-gated open-source models on your computer.**
   
   - **Using Replicate**  

     Sign up and obtain an API key from [Replicate](https://replicate.com/account/api-tokens).
     
5. **Set Environment Variables**

   Set your API key for the framework you intend to use (that is - either OpenRouter, or OpenAI, or HuggingFace) in the environment variable (Note, that if you set up more than one framework API key, the framework choice will obey the priority order below):

   - **Using OpenRouter**

   ```bash
   export OPENROUTER_API_KEY='your_openrouter_api_key_here'
   ```
   
   - **Using OpenAI**

   ```bash
   export OPENAI_API_KEY='your_openai_api_key_here'
   # Optionally, you can set the OpenAI model (the default is gpt-4o) and max tokens (default = 16384)
   export OPENAI_MODEL='o1-preview'
   # Max tokens will be ignored for o1 family of models
   export OPENAI_MAX_TOKENS=8192
   ```

   - **Using HuggingFace**

   ```bash
   # If you plan to deploy a model locally, and if you will deploy only non-gated, open-source type of model, you can replace this key with 'hf_XXX'
   export HF_API_KEY='your_huggingface_api_key_here'
   # Optionally, you can set the HuggingFace model (the default is nvidia/Llama-3.1-Nemotron-70B-Instruct-HF) and max tokens (default = 128000)
   export HF_MODEL='internlm/internlm2_5-20b-chat'
   export HF_MAX_TOKENS=10000
   # If you plan to deploy a model locally, set the base URL. Otherwise, DO NOT SET this environment variable in any circumstance
   export HF_BASE_URL='http://localhost:23333'
   ```
   
   - **Using Replicate**

   ```bash
   export REPLICATE_API_KEY='your_openai_api_key_here'
   # Optionally, you can set the OpenAI model (the default is gpt-4o) and max tokens (default = 16384)
   export REPLICATE_MODEL='meta/meta-llama-3-70b-instruct'
   ```

   **Optional:** If you plan to use the Google Cloud Gemini model for advanced AI capabilities, set up your Google Cloud project and set the following environment variable:

   ```bash
   export GOOGLE_CLOUD_PROJECT='your_google_cloud_project_id'
   ```

   Ensure you have enabled the necessary APIs in your Google Cloud project and have proper authentication set up. Refer to the [Google Cloud documentation](https://cloud.google.com/docs/authentication) for guidance.

6. **Optional: Install python, pip and lmdeploy for local HuggingFace Inference (model deployment on your own computer)**

   If you plan to deploy HuggingFace models for creature's analytic backend via lmdeploy on your computer, you will need to install [python](https://realpython.com/installing-python/) and its package manager [pip](https://pip.pypa.io/en/stable/installation/).  
   For Debian-clone Linux:
   ```bash
   sudo apt-get install -y python3
   wget https://bootstrap.pypa.io/get-pip.py
   python get-pip.py
   # The following line applies to all the OSes
   pip install lmdeploy
   ```

## Installation

1. **Clone the Repository**

   ```bash
   # Clone the repository
   git clone https://github.com/ctapnec/basedAI-Creature.git creature
   cd creature
   ```

2. **Build the Project**

   ```bash
   # Build the project in release mode
   cargo build --release
   ```

   This compiles the project with optimizations, which is recommended for performance.

3. **Optional: Run a HuggingFace model locally with lmdeploy**

   The following is an example on how to locally deploy HuggingFace's internlm/internlm2_5-1_8b-chat model:
   ```bash
   lmdeploy serve api_server internlm/internlm2_5-1_8b-chat --model-name internlm/internlm2_5-1_8b-chat --server-port 23333
   ```
   
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

