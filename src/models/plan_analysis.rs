// MIT License

/*Copyright (c) 2024 Based Labs

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.*/

use crate::models::types::{Plan, PlanStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanAnalysis {
    pub cycle_id: String,
    pub timestamp: DateTime<Utc>,
    pub total_plans: usize,
    pub successful_plans: usize,
    pub failed_plans: usize,
    pub average_score: f64,
    pub best_plan_id: Option<Uuid>,
    pub best_plan_score: f64,
    pub best_plan_summary: String,
}

impl PlanAnalysis {
    pub fn analyze_plans(plans: &[Plan], cycle_id: &str) -> Self {
        let total = plans.len();
        let successful = plans.iter()
            .filter(|p| matches!(p.status, PlanStatus::Completed))
            .count();
        let failed = plans.iter()
            .filter(|p| matches!(p.status, PlanStatus::Failed))
            .count();
            
        let avg_score = if total > 0 {
            plans.iter().map(|p| p.score).sum::<f64>() / total as f64
        } else {
            0.0
        };

        //let best_plan = plans.iter()
        //    .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap());
        // Replacing partial_cmp() with total_cmp() to avoid panic. !!! WARNING !!! - This could mask a bug.
        // Also, total_cmp() treats Nan as the biggest value, so it might not be the best choice.
        let best_plan = plans.iter()
            .max_by(|a, b| a.score.total_cmp(&b.score));

        Self {
            cycle_id: cycle_id.to_string(),
            timestamp: Utc::now(),
            total_plans: total,
            successful_plans: successful,
            failed_plans: failed,
            average_score: avg_score,
            best_plan_id: best_plan.map(|p| p.id),
            best_plan_score: best_plan.map(|p| p.score).unwrap_or(0.0),
            best_plan_summary: best_plan.map(|p| p.summary.clone()).unwrap_or_default(),
        }
    }

    pub fn save_to_file(&self, base_path: &Path) -> std::io::Result<()> {
        let analysis_dir = base_path.join("analysis");
        fs::create_dir_all(&analysis_dir)?;
        
        let filename = format!("analysis_{}.json", self.cycle_id);
        let file_path = analysis_dir.join(filename);
        
        let json = serde_json::to_string_pretty(self)?;
        fs::write(file_path, json)?;
        
        Ok(())
    }
}

pub fn save_plan_to_file(plan: &Plan, base_path: &Path, cycle_id: &str) -> std::io::Result<()> {
    let plans_dir = base_path.join("plans").join(cycle_id);
    fs::create_dir_all(&plans_dir)?;
    
    let filename = format!("plan_{}.json", plan.id);
    let file_path = plans_dir.join(filename);
    
    let json = serde_json::to_string_pretty(plan)?;
    fs::write(file_path, json)?;
    
    Ok(())
}
