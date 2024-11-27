use crate::models::types::{
    CellContext, RealTimeContext, Thought, Plan, PlanNode, PlanNodeStatus, PlanStatus,
    DimensionalPosition,
};
use crate::models::KnowledgeBase;
use chrono::{DateTime, Utc};
use rand::Rng;
use reqwest;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

// Get model name from std::env::var("HF_MODEL") or if not set, use the default model
//static MODEL_IN_USE: &str = std::env::var("HF_MODEL").unwrap_or("nvidia/Llama-3.1-Nemotron-70B-Instruct-HF".to_string());
// Get the tokens from std::env::var("HF_MAX_TOKENS") or if not set, use the default tokens
//static MAX_TOKENS_FOR_MODEL: usize = std::env::var("HF_MAX_TOKENS").unwrap_or("128000".to_string()).parse().unwrap_or(6084);

#[derive(Debug)]
pub enum ClientError {
    LockError(String),
    ApiError(String),
    ParseError(String),
}

impl std::error::Error for ClientError {}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::LockError(msg) => write!(f, "Lock error: {}", msg),
            ClientError::ApiError(msg) => write!(f, "API error: {}", msg),
            ClientError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

struct CachedContext {
    context: RealTimeContext,
    timestamp: SystemTime,
}

struct ContextHistory {
    contexts: VecDeque<RealTimeContext>,
    max_size: usize,
    last_update: DateTime<Utc>,
}

impl Default for ContextHistory {
    fn default() -> Self {
        Self {
            contexts: VecDeque::with_capacity(20),
            max_size: 20,
            last_update: Utc::now(),
        }
    }
}

pub struct HuggingFaceClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    context_cache: Arc<Mutex<Option<CachedContext>>>,
    context_history: Arc<Mutex<ContextHistory>>,
    knowledge_base: Arc<Mutex<Option<KnowledgeBase>>>,
}

impl HuggingFaceClient {
    pub fn new(api_key: String) -> Result<Self, Box<dyn std::error::Error>> {
        if api_key.trim().is_empty() {
            return Err("HF_API_KEY cannot be empty".into());
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(crate::models::constants::API_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Ok(Self {
            client,
            api_key,
            base_url: std::env::var("HF_BASE_URL").unwrap_or("https://api-inference.huggingface.co/models".to_string()),
            context_cache: Arc::new(Mutex::new(None)),
            context_history: Arc::new(Mutex::new(ContextHistory::default())),
            knowledge_base: Arc::new(Mutex::new(None)),
        })
    }

    fn estimate_tokens(text: &str) -> usize {
        text.len() / 4
    }

    fn get_MAX_TOKENS_FOR_MODEL(model: &str) -> usize {
        std::env::var("HF_MAX_TOKENS").unwrap_or("128000".to_string()).parse().unwrap_or(6048)
        /*
        match model {
            _ => 6048,
        }
        */
    }

    async fn get_trending_topics(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        //use crate::utils::logging::*;
        
        let _rng = rand::thread_rng();
        
        // Rate limiting
        static LAST_REQUEST: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let last = LAST_REQUEST.load(std::sync::atomic::Ordering::Relaxed);
        if now - last < 60 { // 1 minute cooldown
            return Ok(vec!["Rate limited - using cached topics".to_string()]);
        }
        
        LAST_REQUEST.store(now, std::sync::atomic::Ordering::Relaxed);
        
        // If self.base_url starts with http://localhost, do not use model name in the url
        
        // Add timeout
        //let url = format!("{}/{}/v1/chat/completions", self.base_url, std::env::var("HF_MODEL").unwrap_or("nvidia/Llama-3.1-Nemotron-70B-Instruct-HF".to_string()));
        let url = if self.base_url.starts_with("http://localhost") {
            format!("{}/v1/chat/completions", self.base_url)
        } else {
            format!(
                "{}/{}/v1/chat/completions",
                self.base_url,
                std::env::var("HF_MODEL").unwrap_or("nvidia/Llama-3.1-Nemotron-70B-Instruct-HF".to_string())
            )
        };
        let mut request = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                    "model": std::env::var("HF_MODEL").unwrap_or("nvidia/Llama-3.1-Nemotron-70B-Instruct-HF".to_string()),
                    "messages": [{
                        "role": "user",
                        "content": r#"
                        Analyze technical developments from the last 72 hours across multiple domains.
                        Focus on posts from accounts with <0.01% following on technical platforms.

                        Required Analysis Vectors:

                        1. TECHNICAL DEVELOPMENTS
                        For each development:
                        EVENT:
                        TIMESTAMP: [Must be within last 72h, exact to minute]
                        SOURCE: 
                        - Repository URL + commit hash
                        - Research paper DOI
                        - Technical blog post URL
                        - System deployment log
                        MENTIONED BY: [Technical accounts only]
                        - Individual researchers (<5k followers)
                        - Research lab accounts
                        - Open source maintainers
                        - System architects
                        - Technical leads
                        TECHNICAL DETAILS:
                        - Implementation specifics
                        - Architecture changes
                        - Performance metrics
                        - Resource requirements
                        - Integration points
                        VALIDATION:
                        - Reproducible results
                        - Test coverage
                        - Benchmark data
                        - Error rates
                        - System logs

                        2. RESEARCH DEVELOPMENTS
                        For each development:
                        PAPER:
                        TIMESTAMP: [Publication/preprint within 72h]
                        DOI/arXiv:
                        AUTHORS:
                        INSTITUTION:
                        KEY FINDINGS:
                        - Methodology
                        - Results
                        - Limitations
                        - Future work
                        VALIDATION:
                        - Experimental setup
                        - Data collection
                        - Statistical analysis
                        - Reproducibility steps

                        3. SYSTEM DEPLOYMENTS
                        For each deployment:
                        SYSTEM:
                        TIMESTAMP: [Deployment within 72h]
                        ORGANIZATION:
                        SCALE:
                        ARCHITECTURE:
                        PERFORMANCE:
                        - Latency metrics
                        - Throughput data
                        - Resource usage
                        - Error rates
                        VALIDATION:
                        - Monitoring logs
                        - Health metrics
                        - Alert history
                        - Recovery data

                        4. TECHNICAL DISCUSSIONS
                        For each significant thread:
                        TOPIC:
                        TIMESTAMP: [Discussion within 72h]
                        PARTICIPANTS: [Technical roles only]
                        KEY POINTS:
                        - Technical challenges
                        - Proposed solutions
                        - Implementation details
                        - Resource considerations
                        VALIDATION:
                        - Code examples
                        - Benchmark results
                        - Test cases
                        - Performance data

                        EVIDENCE REQUIREMENTS:
                        1. Technical Validation
                           - Public repository commits
                           - Published papers
                           - System logs
                           - Performance metrics
                           - Test results
                           - Deployment data

                        2. Source Requirements
                           - Technical accounts only (<0.01% following)
                           - Research institutions
                           - Open source maintainers
                           - System architects
                           - Technical leads
                           - Individual researchers

                        3. Time Constraints
                           - All events within last 72h
                           - Exact timestamps required
                           - Time zone specified
                           - Update frequency noted

                        4. Data Requirements
                                                       - Raw metrics
                           - Benchmark results
                           - Error rates
                           - Resource usage
                           - System logs
                           - Test coverage

                        Return comprehensive analysis of developments from last 72h.
                        Format as structured events with all required fields.
                        Prioritize technical depth over quantity.
                        "#
                    }],
                    "temperature": 0.9,
                    "max_tokens": Self::get_MAX_TOKENS_FOR_MODEL(&std::env::var("HF_MODEL").unwrap_or("nvidia/Llama-3.1-Nemotron-70B-Instruct-HF".to_string()))
            }));
        
        if !self.base_url.starts_with("http://localhost") {
            request = request.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let response = request.send().await?;
        let json: serde_json::Value = response.json().await?;

        //log_info(&format!("Trending topics response: {:?}", json));

        let response_text = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
            
        // Parse and extract events with additional validation
        let mut events = Vec::new();
        let mut current_event = String::new();
        let mut in_event = false;
        
        for line in response_text.lines() {
            let line = line.trim();
            if line.starts_with("EVENT:") {
                if !current_event.is_empty() {
                    events.push(current_event.trim().to_string());
                }
                current_event = line.trim_start_matches("EVENT:").trim().to_string();
                in_event = true;
            } else if in_event && !line.is_empty() {
                current_event.push_str("
");
                current_event.push_str(line);
            }
        }
        
        if !current_event.is_empty() {
            events.push(current_event.trim().to_string());
        }

        Ok(events)
    }

    pub async fn gather_real_time_context(
        &self,
        cell_thoughts: Option<Vec<String>>,
    ) -> Result<RealTimeContext, Box<dyn std::error::Error>> {
        let should_refresh = {
            match self.context_cache.lock() {
                Ok(cache) => {
                    match cache.as_ref() {
                        Some(cached) => cached.timestamp.elapsed()
                            .map_or(true, |d| d >= Duration::from_secs(300)),
                        None => true
                    }
                }
                Err(e) => {
                    eprintln!("Cache lock error: {}", e);
                    true
                }
            }
        };

        if !should_refresh {
            return Ok(RealTimeContext::default());
        }

        // Get previous contexts for comparison
        let previous_contexts = {
            let history = self.context_history.lock().unwrap();
            history.contexts.iter().cloned().collect::<Vec<_>>()
        };

        let trending_topics = self.get_trending_topics().await?;

        // Deduplicate and filter topics
        let _unique_topics: Vec<String> = trending_topics.clone()
            .into_iter()
            .filter(|topic| {
                let topic_lower = topic.to_lowercase();
                !previous_contexts.iter().any(|ctx| 
                    ctx.market_trends.iter().any(|t| t.to_lowercase() == topic_lower) ||
                    ctx.technological_developments.iter().any(|t| t.to_lowercase() == topic_lower) ||
                    ctx.current_events.iter().any(|t| t.to_lowercase() == topic_lower)
                )
            })
            .collect();

        let thoughts_context = if let Some(thoughts) = cell_thoughts {
            format!(
                "
Recent colony thoughts:
{}",
                thoughts
                    .iter()
                    .map(|t| format!("- {}", t))
                    .collect::<Vec<_>>()
                    .join("
")
            )
        } else {
            String::new()
        };

        let context_query = format!(
            r#"
            System Analysis Framework:

            CONTEXT:
            {}
            {}

            1. POWER DYNAMICS ANALYSIS
            - Disrupted hierarchies
            - Emergent control mechanisms
            - Resource flow shifts
            - Influence network changes

            2. SYSTEM BOUNDARIES ANALYSIS
            - Interface mutations
            - Boundary dissolutions
            - Unexpected connections
            - Integration points

            3. TEMPORAL PATTERNS ANALYSIS
            - Evolution trajectories
            - Decay patterns
            - Cyclic behaviors
            - Timescale interactions

            4. EMERGENCE ANALYSIS
            - Unexpected properties
            - Feedback loops
            - Pattern formation
            - System surprises

            5. ASSUMPTION ANALYSIS
            - Questionable constraints
            - Hidden potentials
            - Artificial limitations
            - Missed connections

            Required per analysis vector:
            1. Evidence Trail:
               - Active repository commits (72h)
               - Researcher activities (72h)
               - Experiment results (72h)
               - Deployment metrics (72h)

            2. Power Implications:
               - Control shifts
               - Resource reallocations
               - Relationship changes
               - Influence flows

            3. System Effects:
               - Boundary changes
               - Interface formations
               - Pattern emergences
               - Capability evolutions

            4. Technical Details:
               - Architecture diagrams
               - Integration points
               - Data flows
               - Control mechanisms

            Format as:
            VECTOR: [Analysis Type]
            CONVENTIONAL VIEW: [Standard interpretation]
            RADICAL INSIGHT: [Non-obvious observation]
            EVIDENCE: [Concrete proof points]
            IMPLICATIONS: [Cascading effects]
            DIAGRAM: [ASCII representation]

            Categories labeled and structured hierarchically.
        "#,
            trending_topics
                .iter()
                .map(|t| format!("- {}", t))
                .collect::<Vec<_>>()
                .join("
"),
            thoughts_context
        );

        let response = self.query_llm(&context_query).await?;
        let parsed = self.parse_context_response(&response)?;

        let mut context = RealTimeContext {
            timestamp: Utc::now(),
            market_trends: parsed.get("market_trends").cloned().unwrap_or_default(),
            current_events: parsed.get("current_events").cloned().unwrap_or_default(),
            technological_developments: parsed
                .get("technological_developments")
                .cloned()
                .unwrap_or_default(),
            user_interactions: parsed.get("user_interactions").cloned().unwrap_or_default(),
            environmental_data: HashMap::new(),
            mission_progress: Vec::new(),
        };

        // Update cache with timestamp check
        let now = SystemTime::now();
        let mut cache = self.context_cache.lock().unwrap();
        if let Some(cached) = cache.as_ref() {
            if cached.timestamp.elapsed().unwrap_or_default() > Duration::from_secs(300) {
                *cache = None;
            }
        }
        *cache = Some(CachedContext {
            context: context.clone(),
            timestamp: now,
        });

        // Update history with timestamp and better deduplication
        {
            let mut history = self.context_history.lock().map_err(|e| format!("History lock error: {}", e))?;
            
            // Clean up old contexts first
            let cutoff = Utc::now() - chrono::Duration::hours(24);
            while history.contexts.front().map(|c| c.timestamp < cutoff).unwrap_or(false) {
                history.contexts.pop_front();
            }
            
            // Deduplicate new context before adding
            let mut seen_topics = std::collections::HashSet::new();
            let mut clean_context = context.clone();
            
            clean_context.market_trends.retain(|t| seen_topics.insert(normalize_topic(t)));
            clean_context.technological_developments.retain(|t| seen_topics.insert(normalize_topic(t)));
                            clean_context.current_events.retain(|t| seen_topics.insert(normalize_topic(t)));
            clean_context.user_interactions.retain(|t| seen_topics.insert(normalize_topic(t)));
            
            history.contexts.push_back(clean_context.clone());
            history.last_update = Utc::now();
            
            // Enforce max size
            while history.contexts.len() > history.max_size {
                history.contexts.pop_front();
            }
            
            context = clean_context; // Use deduplicated context
        }

        Ok(context)
    }

    pub async fn generate_contextual_thought(
        &self,
        cell_context: &CellContext,
        real_time_context: &RealTimeContext,
        colony_mission: &str,
    ) -> Result<(String, f64, Vec<String>), Box<dyn std::error::Error>> {
        let mut results = self.generate_contextual_thoughts_batch(&[(Uuid::new_v4(), cell_context)], real_time_context, colony_mission, &[]).await?;
        
        // Take the first result from the batch
        if let Some((_, thoughts)) = results.iter().next().map(|(k,v)| (k,v.clone())) {
            if let Some(thought) = thoughts.into_iter().next() {
                return Ok(thought);
            }
        }
        
        Err("Failed to generate thought".into())
    }

    pub async fn generate_contextual_thoughts_batch(
        &self,
        cell_contexts: &[(Uuid, &CellContext)],
        real_time_context: &RealTimeContext,
        colony_mission: &str,
        recent_thoughts: &[Thought],
    ) -> Result<HashMap<Uuid, Vec<(String, f64, Vec<String>)>>, Box<dyn std::error::Error>> {
        let sub_batch_size = 3;
        let mut all_results = HashMap::new();

        // Format recent thoughts context
        let recent_thoughts_context = if !recent_thoughts.is_empty() {
            format!("
RECENT COLONY THOUGHTS:
{}", 
                recent_thoughts.iter()
                    .take(10)
                    .map(|t| format!("- {}", t.content))
                    .collect::<Vec<_>>()
                    .join("
"))
        } else {
            String::new()
        };

        for chunk in cell_contexts.chunks(sub_batch_size) {
            let kb_context = if let Some(kb) = self.knowledge_base.lock().unwrap().as_ref() {
                format!("
Knowledge Base Context:
{}", kb.compressed_content)
            } else {
                String::new()
            };

            let cell_states = chunk
                .iter()
                .map(|(id, ctx)| {
                    format!(
                        "### CELL {}
FOCUS: {}
ENERGY: {}
",
                        id,
                        ctx.current_focus,
                        ctx.energy_level
                    )
                })
                .collect::<Vec<_>>()
                .join("
");

            let context_prompt = format!(
                r#"Contextual Analysis Framework:

                CURRENT OBJECTIVE: {}
                KNOWLEDGE CONTEXT: {}

                REAL-TIME CONTEXT:
                Market Trends: {}
                Tech Developments: {}
                Current Events: {}
                User Interactions: {}

                CELL STATES:
                {}

                THOUGHT GENERATION RULES:
                1. DO NOT DUPLICATE OR CLOSELY MIRROR ANY RECENT COLONY THOUGHTS
                2. Each thought must explore completely new territory
                3. If a thought seems similar to an existing one, take a radically different approach
                4. Build upon but transform existing insights into novel directions
                5. Explicitly avoid repeating:
                   - Core concepts from recent thoughts
                   - Similar conclusions or recommendations
                   - Related technical approaches
                   - Parallel solution paths

                ENVIRONMENTAL SIGNALS:
                1. SYSTEM DYNAMICS
                   - Emerging patterns
                   - Network effects
                   - Adaptation signals
                   - Behavioral shifts

                2. EVOLUTIONARY VECTORS
                   - Development paths
                   - Growth patterns
                   - Adaptation cycles
                   - Scale dynamics

                3. EMERGENCE INDICATORS
                   - Novel properties
                   - Pattern genesis
                   - System innovations

                4. BOUNDARY ANALYSIS
                   - Current limits
                   - Growth potential
                   - System constraints
                   - Connection opportunities

                ENTITY STATES:
                {}

                Required Format (repeat for each cell):

                ### CELL <uuid>

                THOUGHT STRUCTURE:
                1. OBSERVATION
                   - Current state
                   - Key patterns
                   - System dynamics

                2. ANALYSIS
                   - Conventional wisdom
                   - Hidden assumptions
                   - Unexpected connections
                   - Evidence trail

                3. SYNTHESIS
                   - Novel perspective
                   - Strategic implications
                   - Cascading effects
                   - Action vectors

                4. VISUALIZATION
                   [ASCII representation showing:]
                   - Core components
                   - Dependencies
                   - Data flows
                   - Integration points
                   - System boundaries
                   - Feedback loops

                Example format:
                                                    [Core Goal]
                                                        |
                                    +-------------------+-------------------+
                                    |                   |                   |
                            [Component A]        [Component B]        [Component C]
                                |                     |                    |
                        +-------+-------+      +------+------+     +------+------+
                        |       |       |      |      |      |     |      |      |
                    [Task 1] [Task 2] [Task 3] ...  ...    ...   ...    ...    ...

                Use box drawing characters: ─ │ ┌ ┐ └ ┘ ├ ┤ ┬ ┴ ┼

                THOUGHT: [Core insight challenging assumptions] (500+ words)
                RELEVANCE: <0.0-1.0>
                FACTORS: [Exactly 3 key factors]"#,
                colony_mission,
                kb_context,
                real_time_context
                    .market_trends
                    .first()
                    .unwrap_or(&String::new()),
                real_time_context
                    .technological_developments
                    .first()
                    .unwrap_or(&String::new()),
                real_time_context
                    .current_events
                    .first()
                    .unwrap_or(&String::new()),
                real_time_context
                    .user_interactions
                    .first()
                    .unwrap_or(&String::new()),
                cell_states,
                cell_states  // Added this for ENTITY STATES section
            );

            println!("
║ Processing sub-batch of {} cells", chunk.len());
            match tokio::time::timeout(
                std::time::Duration::from_secs(100),
                self.query_llm(&context_prompt),
            )
            .await
            {
                Ok(Ok(response)) => {
                    if let Ok(results) = self.parse_batch_thought_response(&response) {
                        println!("║ Generated {} thoughts", results.len() * 3);
                        for (id, thoughts) in &results {
                            println!("║");
                            println!("║ Cell {}", id);
                            for (i, (thought, score, factors)) in thoughts.iter().enumerate() {
                                println!("║ ├─ Thought {}", i + 1);
                                println!("║ │  Score: {:.2}", score);
                                println!("║ │  Factors:");
                                for factor in factors {
                                    println!("║ │    - {}", factor);
                                }
                                println!("║ │  Content: {:.100}...", thought);
                            }
                        }
                        all_results.extend(results);
                    } else {
                        eprintln!("Failed to parse results from response:
{}", response);
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("Error in sub-batch: {}", e);
                }
                Err(_) => {
                    eprintln!("Timeout in sub-batch 100s");
                }
            }
        }

        Ok(all_results)
    }

    pub async fn create_plan(
        &self,
        thoughts: &[Thought],
    ) -> Result<Plan, Box<dyn std::error::Error>> {
        let chunk_size = 5;
        let thought_chunks: Vec<Vec<Thought>> = thoughts
            .chunks(chunk_size)
            .map(|chunk| chunk.iter().cloned().collect())
            .collect();

        let mut consolidated_plans = Vec::new();

        for chunk in thought_chunks {
            let thoughts_context = chunk
                .iter()
                .map(|t| format!("- {}", t.content))
                .collect::<Vec<_>>()
                .join("
");

            let chunk_plan = self
                .query_llm(&format!(
                    r#"System Evolution Framework:

    CONTEXT SIGNALS:
    {}

    Analyze each vector as a complex adaptive system:
    1. NETWORK DYNAMICS
    - Flow patterns
    - Resource distribution
    - Connection topology
    - Interaction modes

    2. BOUNDARY CONDITIONS
    - Interface dynamics
    - Connection patterns
    - Integration vectors
    - Barrier dissolution

    3. EMERGENCE PATTERNS
    - Novel properties
    - Feedback cycles
    - Pattern evolution
    - System adaptations

    4. POTENTIAL SPACES
    - Unexplored capabilities
    - Constraint removal
    - Novel applications

    Required Format:

    SUMMARY: [Comprehensive system overview]

    For each component:
    COMPONENT: [Name]
    CONVENTIONAL VIEW: [Standard approach]
    RADICAL SHIFT: [New possibility]
    EVIDENCE: [Proof points]
        IMPLEMENTATION:
    - Technical specifications
    - Resource requirements
    - Timeline estimates
    - Success metrics
    - Risk assessment

    ARCHITECTURE:
    [Detailed ASCII mind map showing:]
    - Core components
    - Dependencies
    - Data flows
    - Integration points
    - System boundaries
    - Feedback loops
    
    Example format:
                                    [Core Goal]
                                        |
                    +-------------------+-------------------+
                    |                   |                   |
            [Component A]        [Component B]        [Component C]
                |                     |                    |
        +-------+-------+      +------+------+     +------+------+
        |       |       |      |      |      |     |      |      |
    [Task 1] [Task 2] [Task 3] ...  ...    ...   ...    ...    ...
    
    Use box drawing characters: ─ │ ┌ ┐ └ ┘ ├ ┤ ┬ ┴ ┼

    INTEGRATION POINTS:
    - System connections
    - Data flows
    - Control mechanisms
    - Feedback loops"#,
                    thoughts_context
                ))
                .await?;
            

            consolidated_plans.push(chunk_plan);
        }

        let combined_plan = self
            .query_llm(&format!(
                r#"System Integration Framework:

    COMPONENT PLANS:
    {}

    1. POWER DYNAMICS ANALYSIS
    - {{Control mechanisms}}
    - {{Resource flows  }}
    - {{Influence networks}} 
    - {{Authority structures}}

    2. SYSTEM BOUNDARIES ANALYSIS
    - {{Interface points}}
    - {{Connection patterns}}
    - {{Integration opportunities}}
    - {{Boundary dissolutions}}

    3. EMERGENCE VECTORS ANALYSIS
    - {{Unexpected properties}}
    - {{Feedback loops}}
    - {{Pattern formation}}
    - {{System surprises}}

    4. HIDDEN POTENTIALS ANALYSIS
    - {{Untapped capabilities}}
    - {{Novel applications}}
    - {{Constraint removals}}
    - {{Integration possibilities}}

    Required Format:

    MASTER PLAN:
    [2000+ word comprehensive integration]

    For each integration point:
    INTERFACE: [Name]
    CONVENTIONAL VIEW: [Standard approach]
    RADICAL SHIFT: [New possibility]
    EVIDENCE: [Proof points]
    IMPLICATIONS: [Cascading effects]

    ARCHITECTURE:
    [Detailed system architecture in ASCII]

    Requirements:
    1. Logical component flow
    2. Clear dependencies
    3. Measurable outcomes
    4. Risk mitigations
    5. Resource allocations"#,
                consolidated_plans.join("

=== Next Component ===

")
            ))
            .await?;

        let enhanced_plan = self
            .query_llm(&format!(
                r#"Technical Integration Analysis Framework:

    BASE PLAN:
    {}

    1. POWER DYNAMICS VECTORS
    - Control shifts
    - Resource reallocations
    - Influence flows
    - Authority transitions

    2. SYSTEM BOUNDARIES
    - Interface mutations
    - Connection formations
    - Integration emergences 
    - Boundary dissolutions

    3. TEMPORAL PATTERNS
    - Evolution trajectories
    - Decay patterns
    - Cyclic behaviors
    - Timescale interactions

    4. EMERGENT PROPERTIES
    - Unexpected capabilities
    - Feedback loops
    - Pattern formations
    - System surprises

    Required Technical Analysis:
    1. Recent Developments (72h)
       - Commit activities
       - Research publications
       - Experiment results
       - Deployment metrics

    2. Integration Points
       - System interfaces
       - Data flows
       - Control mechanisms
       - Feedback systems

    3. Resource Requirements
       - Computational needs
       - Storage demands
       - Network capacity
       - Processing power

    4. Performance Metrics
       - Response times
       - Throughput rates
       - Error margins
       - Recovery speeds

    Format each component with:
    COMPONENT: [Name]
    TECHNICAL_BASELINE: [Current state]
    ENHANCEMENT_VECTOR: [Improvement path]
    EVIDENCE: [Proof points]
    METRICS: [Success measures]

    Requirements:
    1. 2000+ words
    2. RFP structure
    3. Technical precision
    4. Implementation focus"#,
                combined_plan
            ))
            .await?;

        let mut nodes = Vec::new();
        let mut current_node = None;
        let mut summary = String::new();
        let mut in_summary = false;
        let mut in_components = false;

        // Ensure we have at least one default node if none are parsed
        let default_node = PlanNode {
            id: Uuid::new_v4(),
            title: "Initial System Analysis".to_string(),
            description: "Analyze current system state and identify key improvement vectors".to_string(),
            dependencies: Vec::new(),
            estimated_completion: 0.2,
            status: PlanNodeStatus::Pending,
        };
        nodes.push(default_node);

        for line in enhanced_plan.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with("SUMMARY:") {
                in_summary = true;
                summary = line.trim_start_matches("SUMMARY:").trim().to_string();
            } else if in_summary && !line.starts_with("COMPONENTS:") {
                summary.push_str(" ");
                summary.push_str(line);
            } else if line.starts_with("COMPONENTS:") || 
                      line.starts_with("IMPLEMENTATION:") ||
                      line.starts_with("TECHNICAL COMPONENTS:") {
                in_summary = false;
                in_components = true;
            } else if in_components {
                if line.starts_with(|c: char| c.is_digit(10)) || 
                   line.starts_with("COMPONENT:") ||
                   line.starts_with("INTERFACE:") {
                    if let Some(node) = current_node {
                        nodes.push(node);
                    }

                    let title = line
                        .trim_start_matches(|c: char| c.is_digit(10) || c == '.' || c == ':')
                        .trim_start_matches("COMPONENT")
                        .trim_start_matches("INTERFACE")
                        .trim()
                        .to_string();

                    current_node = Some(PlanNode {
                        id: Uuid::new_v4(),
                        title,
                        description: String::new(),
                        dependencies: Vec::new(),
                        estimated_completion: rand::thread_rng().gen_range(0.2..0.4),
                        status: PlanNodeStatus::Pending,
                    });
                } else if (line.starts_with('-') || line.starts_with("DESCRIPTION:")) && current_node.is_some() {
                    if let Some(ref mut node) = current_node {
                        if !node.description.is_empty() {
                            node.description.push_str("
");
                        }
                        node.description.push_str(line
                            .trim_start_matches('-')
                            .trim_start_matches("DESCRIPTION:")
                            .trim());
                    }
                }
            }
        }

        if let Some(node) = current_node {
            nodes.push(node);
        }

        if nodes.len() == 1 && !summary.is_empty() {
            for line in summary.lines() {
                if line.contains(':') || line.contains('-') {
                    nodes.push(PlanNode {
                        id: Uuid::new_v4(),
                        title: line.split(':').next().unwrap_or(line).trim().to_string(),
                        description: line.split(':').nth(1).unwrap_or("").trim().to_string(),
                        dependencies: Vec::new(),
                        estimated_completion: rand::thread_rng().gen_range(0.2..0.4),
                        status: PlanNodeStatus::Pending,
                    });
                }
            }
        }

        let score = if !nodes.is_empty() {
            nodes.iter().map(|n| n.estimated_completion).sum::<f64>() / nodes.len() as f64
        } else {
            0.0
        };

        Ok(Plan {
            id: Uuid::new_v4(),
            thoughts: thoughts.to_vec(),
            nodes,
            summary,
            score,
            participating_cells: Vec::new(),
            created_at: Utc::now(),
            status: PlanStatus::Proposed,
        })
    }

    pub async fn evaluate_dimensional_state(
        &self,
        dimensional_position: &DimensionalPosition,
        recent_thoughts: &[Thought],
        recent_plans: &[Plan],
    ) -> Result<(f64, f64), Box<dyn std::error::Error>> {
        let thoughts_context = recent_thoughts
            .iter()
            .map(|t| format!("- {}", t.content))
            .collect::<Vec<_>>()
            .join("
");

        let plans_context = recent_plans
            .iter()
            .map(|p| format!("- {}: {}", p.id, p.summary))
            .collect::<Vec<_>>()
            .join("
");

        let eval_prompt = format!(
            r#"Dimensional Analysis Framework:

    CURRENT STATE:
    Emergence: {:.2}
    Coherence: {:.2}
    Resilience: {:.2}
    Intelligence: {:.2}
    Efficiency: {:.2}
    Integration: {:.2}

    CONTEXT:
    THOUGHTS:
    {}

    PLANS:
    {}

    Analysis Vectors:

    Fill out each of the items on here in depth just like a CEO applies to take a company public.
    1. POWER DYNAMICS ANALYSIS
    - {{Control mechanisms}}
    - {{Resource flows  }}
    - {{Influence networks}} 
    - {{Authority structures}}

    2. SYSTEM BOUNDARIES ANALYSIS
    - {{Interface points}}
    - {{Connection patterns}}
    - {{Integration opportunities}}
    - {{Boundary dissolutions}}

    3. EMERGENCE VECTORS ANALYSIS
    - {{Unexpected properties}}
    - {{Feedback loops}}
    - {{Pattern formation}}
    - {{System surprises}}

    4. HIDDEN POTENTIALS ANALYSIS
    - {{Untapped capabilities that could result from this plan}}
    - Removed constraints
    - {{Novel applications of the plans}}

    Required Analysis Format:

    DIMENSIONAL_SCORES:
    For each dimension:
    DIMENSION: [Name]
    CONVENTIONAL VIEW: [Standard assessment]
    RADICAL INSIGHT: [Non-obvious observation]
    EVIDENCE: [Concrete proof points]
    SCORE: <-100 to 100>
    IMPLICATIONS: [Cascading effects]

    ENERGY_DYNAMICS:
    CURRENT_STATE: [Assessment]
    SHIFT_VECTOR: [Direction]
    MAGNITUDE: <-100 to 100>
    EVIDENCE: [Proof points]

    DOPAMINE_DYNAMICS:
    ENGAGEMENT_PATTERN: [Assessment]
    REINFORCEMENT_VECTOR: [Direction]
    MAGNITUDE: <0.0 to 1.0>
    EVIDENCE: [Proof points]

    Requirements:
    1. Evidence-based scoring
    2. Multi-framework analysis
    3. Pattern recognition
    4. Emergence identification
    5. System dynamics mapping"#,
            dimensional_position.emergence,
            dimensional_position.coherence,
            dimensional_position.resilience,
            dimensional_position.intelligence,
            dimensional_position.efficiency,
            dimensional_position.integration,
            thoughts_context,
            plans_context
        );

        let response = self.query_llm(&eval_prompt).await?;

        let mut energy_impact = 0.0;
        let mut dopamine_impact = 0.5;

        for line in response.lines() {
            let line = line.trim();
            if line.starts_with("MAGNITUDE:") {
                if energy_impact == 0.0 {
                    energy_impact = line
                        .trim_start_matches("MAGNITUDE:")
                        .trim()
                        .parse()
                        .unwrap_or(0.0);
                } else {
                    dopamine_impact = line
                        .trim_start_matches("MAGNITUDE:")
                        .trim()
                        .parse()
                        .unwrap_or(0.5);
                }
            }
        }

        Ok((energy_impact, dopamine_impact))
    }

    pub async fn compress_memories(
        &self,
        memories: &[String],
    ) -> Result<String, Box<dyn std::error::Error>> {
        let memories_text = memories.join("
");
        let prompt = format!(
            r#"Memory Compression Framework:

    CONTENT:
    {}

    Analysis Vectors:
    1. CORE PATTERNS
       - Recurring themes
       - Common elements
       - Shared structures

    2. RELATIONSHIP MAPPING
       - Direct connections
       - Indirect links
       - Hidden dependencies

    3. KNOWLEDGE SYNTHESIS
       - Key insights
       - Critical learnings
       - Fundamental principles

    4. STRATEGIC IMPORTANCE
       - Action triggers
       - Decision points
       - Resource implications

    Required Format:
    PATTERN: [Identified pattern]
    EVIDENCE: [Supporting data]
    IMPLICATIONS: [Strategic impact]
    ACTIONABILITY: [Implementation path]

    Organize by:
    1. Impact magnitude
    2. Implementation feasibility
    3. Resource requirements
    4. Time sensitivity

    Focus on:
    1. Pattern emergence
    2. Strategic relevance
    3. Actionable insights
    4. Critical dependencies"#,
            memories_text
        );

        self.query_llm(&prompt).await
    }

    async fn compress_knowledge(
        &self,
        content: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let prompt = format!(
            r#"Knowledge Synthesis Framework:

    CONTENT:
    {}

    Analysis Vectors:
    1. CORE CONCEPTS
       - Fundamental principles
       - Key methodologies
       - Critical patterns

    2. RELATIONSHIP MAPPING
       - Direct connections
       - Indirect links
       - Hidden dependencies

    3. PATTERN RECOGNITION
       - Recurring themes
       - Common structures
       - Shared elements

    4. STRATEGIC RELEVANCE
       - Action implications
       - Decision impacts
       - Resource requirements

    Required Format:
    CONCEPT: [Core concept]
    EVIDENCE: [Supporting data]
    CONNECTIONS: [Related elements]
    IMPLICATIONS: [Strategic impact]

    Organize by:
    1. Impact magnitude
    2. Implementation relevance
    3. Resource implications
    4. Time sensitivity

    Preserve:
    1. Technical accuracy
    2. Source references
    3. Critical details
    4. Implementation paths"#,
            content
        );

        self.query_llm(&prompt).await
    }

    pub async fn initialize_knowledge_base(&self) -> Result<(), Box<dyn std::error::Error>> {
        let files = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            async { KnowledgeBase::load_files("knowledgebase") }
        ).await??;

        if files.is_empty() {
            println!("No knowledge base files found in knowledgebase directory");
            return Ok(());
        }

        println!("Loading {} knowledge base files...", files.len());

        let combined_content = files
            .iter()
            .map(|(name, content)| format!("File: {}
{}
", name, content))
            .collect::<Vec<_>>()
            .join("
---
");
                match self.compress_knowledge(&combined_content).await {
            Ok(compressed) => {
                let file_count = files.len();
                let kb = KnowledgeBase {
                    compressed_content: compressed,
                    last_updated: Utc::now(),
                    source_files: files.into_iter().map(|(name, _)| name).collect(),
                };

                if let Ok(mut guard) = self.knowledge_base.lock() {
                    *guard = Some(kb);
                    println!("Knowledge base initialized with {} files", file_count);
                    Ok(())
                } else {
                    Err("Failed to acquire knowledge base lock".into())
                }
            }
            Err(e) => Err(format!("Failed to compress knowledge: {}", e).into())
        }
    }

    pub async fn query_llm(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        //use crate::utils::logging::*;
        
        //let url = format!("{}/{}/v1/chat/completions", self.base_url, std::env::var("HF_MODEL").unwrap_or("nvidia/Llama-3.1-Nemotron-70B-Instruct-HF".to_string()));
        let url = if self.base_url.starts_with("http://localhost") {
            format!("{}/v1/chat/completions", self.base_url)
        } else {
            format!(
                "{}/{}/v1/chat/completions",
                self.base_url,
                std::env::var("HF_MODEL").unwrap_or("nvidia/Llama-3.1-Nemotron-70B-Instruct-HF".to_string())
            )
        };

        //log_info(&format!("Querying LLM with URL: {}", url));

        let mut request = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": std::env::var("HF_MODEL").unwrap_or("nvidia/Llama-3.1-Nemotron-70B-Instruct-HF".to_string()),
                "messages": [{
                    "role": "user",
                    "content": prompt
                }],
                "temperature": 0.7,
                "max_tokens": Self::get_MAX_TOKENS_FOR_MODEL(&std::env::var("HF_MODEL").unwrap_or("nvidia/Llama-3.1-Nemotron-70B-Instruct-HF".to_string())),
            }));
        
        if !self.base_url.starts_with("http://localhost") {
            request = request.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let response = request.send().await?;
        //log_info(&format!("LLM response: {:?}", response));
        let json: serde_json::Value = response.json().await?;

        //log_info(&format!("LLM JSON: {:?}", json));

        Ok(json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }

    fn parse_context_response(
        &self,
        response: &str,
    ) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
        let mut result = HashMap::new();
        let mut current_category = String::new();
        let mut current_items = Vec::new();

        for line in response.lines() {
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            if line.ends_with(':') {
                if !current_category.is_empty() {
                    result.insert(current_category.clone(), current_items.clone());
                    current_items.clear();
                }

                current_category = line.trim_end_matches(':').to_lowercase().replace(' ', "_");
            } else if line.starts_with('-') {
                current_items.push(line.trim_start_matches('-').trim().to_string());
            }
        }

        if !current_category.is_empty() {
            result.insert(current_category, current_items);
        }

        Ok(result)
    }

    fn parse_batch_thought_response(
        &self,
        response: &str,
    ) -> Result<HashMap<Uuid, Vec<(String, f64, Vec<String>)>>, Box<dyn std::error::Error>> {
        let mut results = HashMap::new();
        
        // Split into cell sections
        let cell_sections: Vec<&str> = response
            .split(|c| c == '#' || c == '═' || c == '╔' || c == '╚')
            .filter(|s| s.contains("CELL"))
            .collect();

        for section in cell_sections {
            let mut lines = section.lines();
            let mut current_uuid = None;
            let mut current_thought = String::new();
            let mut current_relevance = 0.5;
            let mut current_factors = Vec::new();
            let mut in_thought = false;
            let mut thought_buffer = String::new();

            while let Some(line) = lines.next() {
                let line = line.trim();
                if line.is_empty() { continue; }

                // Extract UUID
                if line.contains("CELL") {
                    if let Some(uuid_str) = line
                        .split(|c: char| !c.is_ascii_hexdigit() && c != '-')
                        .find(|s| s.len() == 36) 
                    {
                        if let Ok(uuid) = Uuid::parse_str(uuid_str) {
                            current_uuid = Some(uuid);
                        }
                    }
                    continue;
                }

                // Parse thought sections and other components
                if line.starts_with("**THOUGHT STRUCTURE:**") {
                    in_thought = true;
                    continue;
                }

                // Handle structured sections
                if line.contains("**OBSERVATION**") || 
                   line.contains("**ANALYSIS**") || 
                   line.contains("**SYNTHESIS**") || 
                   line.contains("**VISUALIZATION**") {
                    if !thought_buffer.is_empty() {
                        thought_buffer.push('\n');
                    }
                    thought_buffer.push_str(line.trim_matches('*').trim());
                    continue;
                }

                // Capture bullet points and content
                if line.starts_with("   - ") || line.starts_with("     - ") {
                    if !thought_buffer.is_empty() {
                        thought_buffer.push('\n');
                    }
                    thought_buffer.push_str(line.trim_start_matches('-').trim());
                    continue;
                }

                // Capture the main thought content
                if line.starts_with("**THOUGHT:**") {
                    in_thought = true;
                    continue;
                }

                // Capture main thought paragraphs, including numbered sections
                if in_thought && !line.starts_with("```") && 
                   !line.contains("RELEVANCE:") && 
                   !line.contains("FACTORS:") {
                    // Handle numbered sections and their content
                    if line.starts_with(|c: char| c.is_digit(10)) || 
                       line.starts_with("**OBSERVATION**") ||
                       line.starts_with("**ANALYSIS**") ||
                       line.starts_with("**SYNTHESIS**") ||
                       line.starts_with("**VISUALIZATION**") {
                        if !thought_buffer.is_empty() {
                            thought_buffer.push('\n');
                        }
                        thought_buffer.push('\n');
                        thought_buffer.push('\n');
                        thought_buffer.push_str(line.trim());
                    } else if line.starts_with("   -") || line.starts_with("     -") {
                        // Handle bullet points with proper indentation
                        if !thought_buffer.is_empty() {
                            thought_buffer.push('\n');
                            thought_buffer.push_str("  "); // Preserve some indentation
                        }
                        thought_buffer.push_str(line.trim_start_matches(|c| c == ' ' || c == '-').trim());
                    } else if !line.is_empty() {
                        // Regular content
                        if !thought_buffer.is_empty() {
                            thought_buffer.push('\n');
                        }
                        thought_buffer.push_str(line.trim());
                    }
                }

                // Handle relevance score at the end of thought
                if line.to_uppercase().contains("RELEVANCE") {
                    if !thought_buffer.is_empty() {
                        current_thought = thought_buffer.clone();
                        thought_buffer.clear();
                    }
                    if let Some(value_str) = line.split(':').nth(1) {
                        if let Some(value) = value_str
                            .trim()
                            .split_whitespace()
                            .next()
                            .and_then(|s| s.parse::<f64>().ok())
                        {
                            current_relevance = value.clamp(0.0, 1.0);
                        }
                    }
                    continue;
                }

                if line.to_uppercase().contains("FACTORS") {
                    if !thought_buffer.is_empty() {
                        current_thought = thought_buffer.clone();
                        thought_buffer.clear();
                    }
                    
                    current_factors = line
                        .split(|c| c == ',' || c == ';' || c == '|')
                        .filter_map(|s| {
                            let cleaned = s.trim()
                                .trim_start_matches(|c| c == '-' || c == '*' || c == '[' || c == ']')
                                .trim_end_matches(|c| c == ']' || c == ':')
                                .trim();
                            if !cleaned.is_empty() {
                                Some(cleaned.to_string())
                            } else {
                                None
                            }
                        })
                        .collect();
                }

                // Collect thought content
                if in_thought && 
                   !line.to_uppercase().contains("DIMENSION") && 
                   !line.to_uppercase().contains("DOPAMINE") &&
                   !line.to_uppercase().contains("RELEVANCE") &&
                   !line.to_uppercase().contains("FACTORS") {
                    if !line.trim().is_empty() {
                        thought_buffer.push_str(line.trim());
                    }
                }
            }

            if !thought_buffer.is_empty() {
                current_thought = thought_buffer.clone();
            }

            if let Some(uuid) = current_uuid {
                if !current_thought.is_empty() {
                                        results.entry(uuid)
                        .or_insert_with(Vec::new)
                        .push((current_thought, current_relevance, current_factors));
                }
            }
        }

        // Improved error reporting
        if results.is_empty() {
            eprintln!("Warning: Failed to parse any thoughts from response");
            eprintln!("Response structure:");
            for (i, line) in response.lines().enumerate() {
                eprintln!("{:3}: {}", i + 1, line);
            }
        }

        Ok(results)
    }
}

fn normalize_topic(topic: &str) -> String {
    topic.trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect()
}
