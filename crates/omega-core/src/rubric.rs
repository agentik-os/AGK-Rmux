//! Rubric generation — define success criteria BEFORE workers start (R-19).
//!
//! The RubricGenerator analyzes mission text and target files to produce
//! a typed Rubric. The oracle writes this to state dir before dispatching workers.

use crate::gate::{CriterionCategory, Rubric, RubricCriterion};
use std::path::Path;

pub struct RubricGenerator;

impl RubricGenerator {
    /// Generate a rubric from mission text and file context.
    pub fn generate(mission_text: &str, files: &[String], _project_dir: &Path) -> Rubric {
        let mut criteria = Vec::new();
        let lower = mission_text.to_lowercase();

        criteria.push(criterion(
            "build-passes",
            "Project builds with zero errors",
            1.0,
            CriterionCategory::Functional,
        ));

        if has_any(
            &lower,
            &["fix", "bug", "broken", "error", "crash", "regression"],
        ) {
            criteria.push(criterion(
                "bug-resolved",
                "The reported bug no longer reproduces",
                1.5,
                CriterionCategory::Functional,
            ));
            criteria.push(criterion(
                "no-side-effects",
                "Fix introduces no new issues in adjacent code",
                1.0,
                CriterionCategory::Quality,
            ));
        }

        if has_any(&lower, &["feature", "add", "implement", "create", "new"]) {
            criteria.push(criterion(
                "feature-functional",
                "New feature works as specified in the mission",
                1.5,
                CriterionCategory::Functional,
            ));
            criteria.push(criterion(
                "edge-cases",
                "Edge cases handled gracefully",
                0.8,
                CriterionCategory::Quality,
            ));
        }

        if has_any(
            &lower,
            &["ui", "ux", "design", "dashboard", "page", "component"],
        ) {
            criteria.push(criterion(
                "visual-correct",
                "UI renders correctly across viewports",
                1.0,
                CriterionCategory::Functional,
            ));
            criteria.push(criterion(
                "no-console-errors",
                "Zero app-origin console errors",
                0.8,
                CriterionCategory::Quality,
            ));
        }

        if has_any(
            &lower,
            &["auth", "security", "login", "password", "token", "secret"],
        ) {
            criteria.push(criterion(
                "security-safe",
                "No security vulnerabilities introduced",
                2.0,
                CriterionCategory::Security,
            ));
        }

        if has_any(
            &lower,
            &["perf", "performance", "speed", "slow", "optimize"],
        ) {
            criteria.push(criterion(
                "perf-target",
                "Performance meets or exceeds stated target",
                1.5,
                CriterionCategory::Performance,
            ));
        }

        for file in files {
            criteria.push(criterion(
                &format!("file-valid-{}", sanitize(file)),
                &format!("{} compiles without errors", file),
                0.5,
                CriterionCategory::Quality,
            ));
        }

        criteria.push(criterion(
            "no-regressions",
            "Existing tests continue to pass",
            1.0,
            CriterionCategory::Quality,
        ));

        Rubric::new(mission_text, criteria)
    }

    /// Generate and persist to state dir before worker dispatch.
    pub fn generate_and_persist(
        mission_text: &str,
        files: &[String],
        project_dir: &Path,
        state_dir: &Path,
        oracle: &str,
    ) -> anyhow::Result<Rubric> {
        let rubric = Self::generate(mission_text, files, project_dir);
        rubric.write(state_dir, oracle)?;
        Ok(rubric)
    }
}

fn criterion(id: &str, desc: &str, weight: f32, cat: CriterionCategory) -> RubricCriterion {
    RubricCriterion {
        id: id.to_string(),
        description: desc.to_string(),
        weight,
        category: cat,
    }
}

fn has_any(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| text.contains(p))
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn generates_build_criterion_always() {
        let rubric = RubricGenerator::generate("do something", &[], &PathBuf::from("."));
        assert!(rubric.criteria.iter().any(|c| c.id == "build-passes"));
    }

    #[test]
    fn bug_fix_adds_criteria() {
        let rubric = RubricGenerator::generate("fix the auth bug", &[], &PathBuf::from("."));
        assert!(rubric.criteria.iter().any(|c| c.id == "bug-resolved"));
        assert!(rubric.criteria.iter().any(|c| c.id == "no-side-effects"));
    }

    #[test]
    fn feature_adds_criteria() {
        let rubric = RubricGenerator::generate("implement new dashboard", &[], &PathBuf::from("."));
        assert!(rubric.criteria.iter().any(|c| c.id == "feature-functional"));
    }

    #[test]
    fn security_adds_high_weight() {
        let rubric = RubricGenerator::generate("fix auth security", &[], &PathBuf::from("."));
        let sec = rubric
            .criteria
            .iter()
            .find(|c| c.id == "security-safe")
            .unwrap();
        assert_eq!(sec.weight, 2.0);
    }

    #[test]
    fn files_add_per_file_criteria() {
        let files = vec!["src/main.rs".to_string(), "src/lib.rs".to_string()];
        let rubric = RubricGenerator::generate("refactor", &files, &PathBuf::from("."));
        assert!(rubric
            .criteria
            .iter()
            .any(|c| c.id.starts_with("file-valid-")));
    }

    #[test]
    fn always_has_no_regressions() {
        let rubric = RubricGenerator::generate("anything", &[], &PathBuf::from("."));
        assert!(rubric.criteria.iter().any(|c| c.id == "no-regressions"));
    }

    #[test]
    fn persists_to_state_dir() {
        let dir = tempfile::tempdir().unwrap();
        let rubric = RubricGenerator::generate_and_persist(
            "fix bug",
            &[],
            dir.path(),
            dir.path(),
            "test-oracle",
        )
        .unwrap();
        assert!(dir.path().join("test-oracle.rubric.json").exists());
        let loaded = Rubric::read(dir.path(), "test-oracle").unwrap().unwrap();
        assert_eq!(loaded.mission, rubric.mission);
    }

    #[test]
    fn ui_work_adds_visual_criteria() {
        let rubric =
            RubricGenerator::generate("redesign the dashboard UI", &[], &PathBuf::from("."));
        assert!(rubric.criteria.iter().any(|c| c.id == "visual-correct"));
        assert!(rubric.criteria.iter().any(|c| c.id == "no-console-errors"));
    }

    #[test]
    fn performance_work_adds_criteria() {
        let rubric = RubricGenerator::generate("optimize page speed", &[], &PathBuf::from("."));
        assert!(rubric.criteria.iter().any(|c| c.id == "perf-target"));
    }
}
