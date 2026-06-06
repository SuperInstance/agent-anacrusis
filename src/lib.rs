//! # agent-anacrusis
//!
//! Pickup notes and preparation for agent systems.
//!
//! In music, an *anacrusis* (or pickup) is a note or phrase that comes before
//! the first strong beat — before the "official" start. It sets up the listener
//! for what's coming. Similarly, the best agents don't wait for a task to
//! officially begin; they start preparing early, warming up, and setting the
//! stage so they can hit the ground running when the downbeat arrives.

use std::collections::HashMap;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// PickupNote
// ---------------------------------------------------------------------------

/// A small action that sets up the main event.
///
/// Like a pickup note in music, a `PickupNote` is a minor action that happens
/// before the main task begins. It primes caches, loads resources, establishes
/// connections, or sets up state so the main action is fast and smooth.
#[derive(Debug, Clone, PartialEq)]
pub enum PickupStatus {
    /// Not yet started.
    Pending,
    /// Currently executing.
    InProgress,
    /// Successfully completed.
    Completed,
    /// Failed to complete.
    Failed(String),
}

/// A small preparatory action before the main task.
#[derive(Debug, Clone)]
pub struct PickupNote {
    /// Unique identifier.
    id: String,
    /// Description of this pickup action.
    description: String,
    /// Estimated time to complete.
    estimated_duration: Duration,
    /// How critical this pickup is (0 = optional, 10 = essential).
    criticality: u8,
    /// Current status.
    status: PickupStatus,
    /// Dependencies (IDs of other pickup notes that must complete first).
    dependencies: Vec<String>,
    /// When execution started.
    started_at: Option<Instant>,
    /// Actual duration.
    actual_duration: Option<Duration>,
    /// Metadata.
    metadata: HashMap<String, String>,
}

impl PickupNote {
    /// Create a new pickup note.
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            estimated_duration: Duration::from_millis(100),
            criticality: 5,
            status: PickupStatus::Pending,
            dependencies: Vec::new(),
            started_at: None,
            actual_duration: None,
            metadata: HashMap::new(),
        }
    }

    /// Set estimated duration.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.estimated_duration = duration;
        self
    }

    /// Set criticality (0–10).
    pub fn with_criticality(mut self, level: u8) -> Self {
        self.criticality = level.min(10);
        self
    }

    /// Add a dependency on another pickup note.
    pub fn depends_on(mut self, id: impl Into<String>) -> Self {
        self.dependencies.push(id.into());
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Mark as in progress.
    pub fn start(&mut self) {
        self.started_at = Some(Instant::now());
        self.status = PickupStatus::InProgress;
    }

    /// Mark as completed.
    pub fn complete(&mut self) {
        if let Some(start) = self.started_at {
            self.actual_duration = Some(start.elapsed());
        }
        self.status = PickupStatus::Completed;
    }

    /// Mark as failed.
    pub fn fail(&mut self, reason: impl Into<String>) {
        if let Some(start) = self.started_at {
            self.actual_duration = Some(start.elapsed());
        }
        self.status = PickupStatus::Failed(reason.into());
    }

    /// Check if this pickup is ready (all dependencies would need external tracking).
    pub fn is_ready(&self) -> bool {
        matches!(self.status, PickupStatus::Pending)
    }

    /// Check if completed successfully.
    pub fn is_completed(&self) -> bool {
        matches!(self.status, PickupStatus::Completed)
    }

    /// Check if failed.
    pub fn is_failed(&self) -> bool {
        matches!(self.status, PickupStatus::Failed(_))
    }

    /// Get the status.
    pub fn status(&self) -> &PickupStatus {
        &self.status
    }

    /// The pickup note ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Estimated duration.
    pub fn estimated_duration(&self) -> Duration {
        self.estimated_duration
    }

    /// Actual duration (if completed or failed).
    pub fn actual_duration(&self) -> Option<Duration> {
        self.actual_duration
    }

    /// Criticality level.
    pub fn criticality(&self) -> u8 {
        self.criticality
    }

    /// Dependencies.
    pub fn dependencies(&self) -> &[String] {
        &self.dependencies
    }

    /// Metadata.
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Is this pickup essential (criticality >= 8)?
    pub fn is_essential(&self) -> bool {
        self.criticality >= 8
    }
}

// ---------------------------------------------------------------------------
// PreparationWindow
// ---------------------------------------------------------------------------

/// Time before main action for warmup/preparation.
///
/// A `PreparationWindow` defines how much lead time an agent has to prepare
/// before the main task starts. Like the beats before the downbeat in an
/// anacrusis, this window is where the agent gets ready.
#[derive(Debug, Clone)]
pub struct PreparationWindow {
    /// Total preparation time available.
    total_time: Duration,
    /// Time already used for preparation.
    used_time: Duration,
    /// When the window opened.
    opened_at: Option<Instant>,
    /// When the main action starts.
    action_starts_at: Option<Instant>,
    /// Pickup notes to execute during this window.
    pickups: Vec<PickupNote>,
    /// Whether the window is active.
    active: bool,
}

impl PreparationWindow {
    /// Create a new preparation window.
    pub fn new(total_time: Duration) -> Self {
        Self {
            total_time,
            used_time: Duration::ZERO,
            opened_at: None,
            action_starts_at: None,
            pickups: Vec::new(),
            active: false,
    }
    }

    /// Open the preparation window.
    pub fn open(&mut self) {
        let now = Instant::now();
        self.opened_at = Some(now);
        self.action_starts_at = Some(now + self.total_time);
        self.active = true;
    }

    /// Add a pickup note to the preparation window.
    pub fn add_pickup(&mut self, pickup: PickupNote) {
        self.pickups.push(pickup);
    }

    /// Get remaining preparation time.
    pub fn remaining(&self) -> Duration {
        self.total_time.saturating_sub(self.used_time)
    }

    /// Get remaining time from wall clock.
    pub fn remaining_wall(&self) -> Option<Duration> {
        self.action_starts_at.map(|t| t.duration_since(Instant::now()))
    }

    /// Record time used for preparation.
    pub fn use_time(&mut self, duration: Duration) {
        self.used_time += duration;
    }

    /// Fraction of preparation time used (0.0–1.0).
    pub fn usage_fraction(&self) -> f64 {
        if self.total_time.is_zero() {
            return 1.0;
        }
        (self.used_time.as_secs_f64() / self.total_time.as_secs_f64()).min(1.0)
    }

    /// Whether the window is still open.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Close the window (main action starts).
    pub fn close(&mut self) {
        self.active = false;
    }

    /// Check if time is running out (< 25% remaining).
    pub fn is_urgent(&self) -> bool {
        self.usage_fraction() > 0.75
    }

    /// Total preparation time.
    pub fn total_time(&self) -> Duration {
        self.total_time
    }

    /// Time used so far.
    pub fn used_time(&self) -> Duration {
        self.used_time
    }

    /// Get all pickup notes.
    pub fn pickups(&self) -> &[PickupNote] {
        &self.pickups
    }

    /// Get pending pickup notes.
    pub fn pending_pickups(&self) -> Vec<&PickupNote> {
        self.pickups.iter().filter(|p| p.is_ready()).collect()
    }

    /// Get completed pickup notes.
    pub fn completed_pickups(&self) -> Vec<&PickupNote> {
        self.pickups.iter().filter(|p| p.is_completed()).collect()
    }

    /// Get failed pickup notes.
    pub fn failed_pickups(&self) -> Vec<&PickupNote> {
        self.pickups.iter().filter(|p| p.is_failed()).collect()
    }

    /// Number of pickup notes.
    pub fn pickup_count(&self) -> usize {
        self.pickups.len()
    }

    /// Check if all essential pickups are complete.
    pub fn essential_pickups_complete(&self) -> bool {
        self.pickups.iter()
            .filter(|p| p.is_essential())
            .all(|p| p.is_completed())
    }

    /// Check if the preparation window is sufficient for remaining pickups.
    pub fn has_sufficient_time(&self) -> bool {
        let remaining_pickup_time: Duration = self.pickups.iter()
            .filter(|p| p.is_ready())
            .map(|p| p.estimated_duration)
            .fold(Duration::ZERO, |acc, d| acc + d);
        remaining_pickup_time <= self.remaining()
    }
}

// ---------------------------------------------------------------------------
// AnticipationDetector
// ---------------------------------------------------------------------------

/// Sense when something is about to start.
///
/// An `AnticipationDetector` watches for signals that a task or event is
/// imminent, allowing the agent to begin preparation (anacrusis) before the
/// official start signal arrives.
#[derive(Debug, Clone)]
pub struct AnticipationSignal {
    /// Signal name.
    name: String,
    /// Strength of the signal (0.0–1.0).
    strength: f64,
    /// Estimated time until the event.
    time_until: Option<Duration>,
    /// Confidence (0.0–1.0).
    confidence: f64,
}

/// Detects when an event is about to happen.
#[derive(Debug, Clone)]
pub struct AnticipationDetector {
    /// Signals that suggest an event is coming.
    signals: Vec<AnticipationSignal>,
    /// Threshold for triggering anticipation.
    threshold: f64,
    /// Whether anticipation is currently active.
    anticipating: bool,
    /// History of signal counts per name.
    signal_history: HashMap<String, u32>,
}

impl AnticipationDetector {
    /// Create a new anticipation detector.
    pub fn new(threshold: f64) -> Self {
        Self {
            signals: Vec::new(),
            threshold: threshold.clamp(0.0, 1.0),
            anticipating: false,
            signal_history: HashMap::new(),
        }
    }

    /// Add an anticipation signal.
    pub fn add_signal(&mut self, signal: AnticipationSignal) {
        *self.signal_history.entry(signal.name.clone()).or_insert(0) += 1;
        self.signals.push(signal);
        self.update_anticipation();
    }

    /// Create a simple signal.
    pub fn signal(name: impl Into<String>, strength: f64) -> AnticipationSignal {
        AnticipationSignal {
            name: name.into(),
            strength: strength.clamp(0.0, 1.0),
            time_until: None,
            confidence: 0.5,
        }
    }

    /// Create a signal with timing info.
    pub fn timed_signal(name: impl Into<String>, strength: f64, time_until: Duration) -> AnticipationSignal {
        AnticipationSignal {
            name: name.into(),
            strength: strength.clamp(0.0, 1.0),
            time_until: Some(time_until),
            confidence: 0.7,
        }
    }

    fn update_anticipation(&mut self) {
        if self.signals.is_empty() {
            self.anticipating = false;
            return;
        }
        let combined = self.combined_strength();
        self.anticipating = combined >= self.threshold;
    }

    /// Compute the combined signal strength (weighted average with confidence).
    pub fn combined_strength(&self) -> f64 {
        if self.signals.is_empty() {
            return 0.0;
        }
        let total_confidence: f64 = self.signals.iter().map(|s| s.confidence).sum();
        if total_confidence == 0.0 {
            return 0.0;
        }
        let weighted: f64 = self.signals.iter()
            .map(|s| s.strength * s.confidence)
            .sum();
        (weighted / total_confidence).clamp(0.0, 1.0)
    }

    /// Is anticipation active?
    pub fn is_anticipating(&self) -> bool {
        self.anticipating
    }

    /// Get the threshold.
    pub fn threshold(&self) -> f64 {
        self.threshold
    }

    /// Number of signals.
    pub fn signal_count(&self) -> usize {
        self.signals.len()
    }

    /// Signal history counts.
    pub fn signal_history(&self) -> &HashMap<String, u32> {
        &self.signal_history
    }

    /// Estimated time until the event (minimum of all timed signals).
    pub fn estimated_time_until(&self) -> Option<Duration> {
        let times: Vec<Duration> = self.signals.iter()
            .filter_map(|s| s.time_until)
            .collect();
        times.into_iter().min()
    }

    /// Clear all signals.
    pub fn clear(&mut self) {
        self.signals.clear();
        self.anticipating = false;
    }
}

// ---------------------------------------------------------------------------
// PreemptiveAction
// ---------------------------------------------------------------------------

/// Act before being asked.
///
/// A `PreemptiveAction` represents something the agent does proactively,
/// anticipating a future need rather than waiting to be told.
#[derive(Debug, Clone)]
pub struct PreemptiveAction {
    /// Unique identifier.
    id: String,
    /// What action to take.
    action: String,
    /// Why we think this action will be needed.
    rationale: String,
    /// Confidence that this action is correct (0.0–1.0).
    confidence: f64,
    /// Cost of the action (resources/time).
    cost: f64,
    /// Benefit if the action is correct.
    benefit: f64,
    /// Cost if the action was wrong (wasted resources).
    waste_cost: f64,
    /// Whether the action has been executed.
    executed: bool,
    /// Whether the action was actually needed.
    was_needed: Option<bool>,
}

impl PreemptiveAction {
    /// Create a new preemptive action.
    pub fn new(id: impl Into<String>, action: impl Into<String>, rationale: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            action: action.into(),
            rationale: rationale.into(),
            confidence: 0.5,
            cost: 1.0,
            benefit: 3.0,
            waste_cost: 0.5,
            executed: false,
            was_needed: None,
        }
    }

    /// Set confidence level.
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set cost/benefit analysis.
    pub fn with_cost_benefit(mut self, cost: f64, benefit: f64, waste_cost: f64) -> Self {
        self.cost = cost;
        self.benefit = benefit;
        self.waste_cost = waste_cost;
        self
    }

    /// Compute the expected value of taking this action.
    pub fn expected_value(&self) -> f64 {
        self.confidence * self.benefit - (1.0 - self.confidence) * self.waste_cost - self.cost
    }

    /// Is this action worth taking? (positive expected value)
    pub fn is_worthwhile(&self) -> bool {
        self.expected_value() > 0.0
    }

    /// Mark as executed.
    pub fn execute(&mut self) {
        self.executed = true;
    }

    /// Mark whether the action was actually needed.
    pub fn resolve(&mut self, was_needed: bool) {
        self.was_needed = Some(was_needed);
    }

    /// Was this action executed?
    pub fn was_executed(&self) -> bool {
        self.executed
    }

    /// Was the action actually needed?
    pub fn was_needed(&self) -> Option<bool> {
        self.was_needed
    }

    /// The action ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The action description.
    pub fn action(&self) -> &str {
        &self.action
    }

    /// The rationale.
    pub fn rationale(&self) -> &str {
        &self.rationale
    }

    /// Confidence.
    pub fn confidence(&self) -> f64 {
        self.confidence
    }

    /// Cost.
    pub fn cost(&self) -> f64 {
        self.cost
    }

    /// Benefit.
    pub fn benefit(&self) -> f64 {
        self.benefit
    }
}

// ---------------------------------------------------------------------------
// SetupVerification
// ---------------------------------------------------------------------------

/// Confirm that preparation was sufficient.
///
/// Before the main action starts, `SetupVerification` checks that all the
/// anacrusis preparation was adequate and the agent is truly ready.
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationResult {
    /// All checks passed; ready to proceed.
    Ready,
    /// Partial readiness; some checks passed, some warnings.
    PartiallyReady {
        warnings: Vec<String>,
    },
    /// Not ready; critical checks failed.
    NotReady {
        failures: Vec<String>,
    },
}

/// A single verification check.
#[derive(Debug, Clone)]
pub struct VerificationCheck {
    /// Check name.
    name: String,
    /// Description of what this check verifies.
    description: String,
    /// Whether the check passed.
    passed: bool,
    /// Optional message (failure reason or success note).
    message: Option<String>,
    /// Whether this is a critical check (failure = not ready).
    critical: bool,
}

impl VerificationCheck {
    /// Create a passing check.
    pub fn pass(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            passed: true,
            message: None,
            critical: false,
        }
    }

    /// Create a failing check.
    pub fn fail(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            passed: false,
            message: Some(reason.into()),
            critical: false,
        }
    }

    /// Mark as critical.
    pub fn critical(mut self) -> Self {
        self.critical = true;
        self
    }

    /// Set description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
}

/// Verify that setup preparation was sufficient.
#[derive(Debug, Clone)]
pub struct SetupVerification {
    /// Checks to perform.
    checks: Vec<VerificationCheck>,
    /// Whether verification has been run.
    verified: bool,
}

impl SetupVerification {
    /// Create a new setup verification.
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
            verified: false,
        }
    }

    /// Add a check.
    pub fn add_check(&mut self, check: VerificationCheck) {
        self.checks.push(check);
        self.verified = false;
    }

    /// Run all checks and return the result.
    pub fn verify(&mut self) -> VerificationResult {
        self.verified = true;
        if self.checks.is_empty() {
            return VerificationResult::Ready;
        }

        let has_critical_failure = self.checks.iter().any(|c| c.critical && !c.passed);
        if has_critical_failure {
            let failures: Vec<String> = self.checks.iter()
                .filter(|c| !c.passed)
                .map(|c| format!("{}: {}", c.name, c.message.as_deref().unwrap_or("failed")))
                .collect();
            return VerificationResult::NotReady { failures };
        }

        let failures: Vec<&VerificationCheck> = self.checks.iter().filter(|c| !c.passed).collect();
        if failures.is_empty() {
            VerificationResult::Ready
        } else {
            let warnings: Vec<String> = failures.iter()
                .map(|c| format!("{}: {}", c.name, c.message.as_deref().unwrap_or("warning")))
                .collect();
            VerificationResult::PartiallyReady { warnings }
        }
    }

    /// Number of checks.
    pub fn check_count(&self) -> usize {
        self.checks.len()
    }

    /// Number of passing checks.
    pub fn passing_count(&self) -> usize {
        self.checks.iter().filter(|c| c.passed).count()
    }

    /// Number of failing checks.
    pub fn failing_count(&self) -> usize {
        self.checks.iter().filter(|c| !c.passed).count()
    }

    /// Whether verification has been run.
    pub fn is_verified(&self) -> bool {
        self.verified
    }

    /// Readiness percentage (0.0–1.0).
    pub fn readiness(&self) -> f64 {
        if self.checks.is_empty() {
            return 1.0;
        }
        self.passing_count() as f64 / self.checks.len() as f64
    }
}

impl Default for SetupVerification {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Anacrusis
// ---------------------------------------------------------------------------

/// The full preparation phase before a main action.
///
/// An `Anacrusis` orchestrates the complete pre-task preparation: the
/// anticipation, the preparation window, the pickup notes, and the final
/// verification. It's the entire "before the downbeat" phase.
#[derive(Debug)]
pub struct Anacrusis {
    /// The main action being prepared for.
    main_action: String,
    /// Preparation window.
    window: PreparationWindow,
    /// Anticipation detector.
    detector: AnticipationDetector,
    /// Setup verification.
    verification: SetupVerification,
    /// Whether the anacrusis phase is complete.
    complete: bool,
}

impl Anacrusis {
    /// Create a new anacrusis for a main action.
    pub fn new(main_action: impl Into<String>, prep_time: Duration) -> Self {
        Self {
            main_action: main_action.into(),
            window: PreparationWindow::new(prep_time),
            detector: AnticipationDetector::new(0.5),
            verification: SetupVerification::new(),
            complete: false,
        }
    }

    /// Add a pickup note.
    pub fn add_pickup(&mut self, pickup: PickupNote) {
        self.window.add_pickup(pickup);
    }

    /// Add a verification check.
    pub fn add_check(&mut self, check: VerificationCheck) {
        self.verification.add_check(check);
    }

    /// Add an anticipation signal.
    pub fn add_signal(&mut self, signal: AnticipationSignal) {
        self.detector.add_signal(signal);
    }

    /// Open the preparation window.
    pub fn begin_preparation(&mut self) {
        self.window.open();
    }

    /// Check if preparation is complete.
    pub fn is_prepared(&self) -> bool {
        self.window.essential_pickups_complete() && self.verification.readiness() >= 1.0
    }

    /// Finalize the anacrusis phase.
    pub fn finalize(&mut self) -> VerificationResult {
        self.complete = true;
        self.window.close();
        self.verification.verify()
    }

    /// The main action.
    pub fn main_action(&self) -> &str {
        &self.main_action
    }

    /// Is the anacrusis complete?
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    /// The preparation window.
    pub fn window(&self) -> &PreparationWindow {
        &self.window
    }

    /// Mutable access to the preparation window.
    pub fn window_mut(&mut self) -> &mut PreparationWindow {
        &mut self.window
    }

    /// The anticipation detector.
    pub fn detector(&self) -> &AnticipationDetector {
        &self.detector
    }

    /// Mutable access to the anticipation detector.
    pub fn detector_mut(&mut self) -> &mut AnticipationDetector {
        &mut self.detector
    }

    /// The verification.
    pub fn verification(&self) -> &SetupVerification {
        &self.verification
    }

    /// Mutable access to verification.
    pub fn verification_mut(&mut self) -> &mut SetupVerification {
        &mut self.verification
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- PickupNote tests ---

    #[test]
    fn test_pickup_note_creation() {
        let pn = PickupNote::new("cache-warm", "Warm the response cache");
        assert_eq!(pn.id(), "cache-warm");
        assert_eq!(pn.description(), "Warm the response cache");
        assert!(pn.is_ready());
        assert!(!pn.is_completed());
        assert_eq!(pn.criticality(), 5);
    }

    #[test]
    fn test_pickup_note_with_options() {
        let pn = PickupNote::new("db-pool", "Initialize DB connection pool")
            .with_duration(Duration::from_secs(2))
            .with_criticality(9)
            .depends_on("config-load")
            .with_metadata("pool_size", "10");
        assert_eq!(pn.estimated_duration(), Duration::from_secs(2));
        assert_eq!(pn.criticality(), 9);
        assert!(pn.is_essential());
        assert_eq!(pn.dependencies(), &["config-load"]);
        assert_eq!(pn.metadata().get("pool_size"), Some(&"10".to_string()));
    }

    #[test]
    fn test_pickup_note_lifecycle() {
        let mut pn = PickupNote::new("test", "Test pickup");
        assert!(pn.is_ready());
        pn.start();
        assert!(!pn.is_ready());
        pn.complete();
        assert!(pn.is_completed());
        assert!(pn.actual_duration().is_some());
    }

    #[test]
    fn test_pickup_note_failure() {
        let mut pn = PickupNote::new("test", "Test pickup");
        pn.start();
        pn.fail("connection refused");
        assert!(pn.is_failed());
        assert!(!pn.is_completed());
        match pn.status() {
            PickupStatus::Failed(reason) => assert_eq!(reason, "connection refused"),
            _ => panic!("Expected Failed status"),
        }
    }

    #[test]
    fn test_pickup_note_essential() {
        let essential = PickupNote::new("e", "essential").with_criticality(8);
        assert!(essential.is_essential());
        let optional = PickupNote::new("o", "optional").with_criticality(3);
        assert!(!optional.is_essential());
    }

    // --- PreparationWindow tests ---

    #[test]
    fn test_preparation_window_creation() {
        let pw = PreparationWindow::new(Duration::from_secs(30));
        assert_eq!(pw.total_time(), Duration::from_secs(30));
        assert_eq!(pw.used_time(), Duration::ZERO);
        assert_eq!(pw.remaining(), Duration::from_secs(30));
        assert!(!pw.is_active());
    }

    #[test]
    fn test_preparation_window_open_close() {
        let mut pw = PreparationWindow::new(Duration::from_secs(30));
        pw.open();
        assert!(pw.is_active());
        pw.close();
        assert!(!pw.is_active());
    }

    #[test]
    fn test_preparation_window_usage() {
        let mut pw = PreparationWindow::new(Duration::from_secs(10));
        assert_eq!(pw.usage_fraction(), 0.0);
        pw.use_time(Duration::from_secs(5));
        assert!((pw.usage_fraction() - 0.5).abs() < 0.01);
        assert_eq!(pw.remaining(), Duration::from_secs(5));
    }

    #[test]
    fn test_preparation_window_urgent() {
        let mut pw = PreparationWindow::new(Duration::from_secs(10));
        assert!(!pw.is_urgent());
        pw.use_time(Duration::from_secs(8));
        assert!(pw.is_urgent());
    }

    #[test]
    fn test_preparation_window_pickups() {
        let mut pw = PreparationWindow::new(Duration::from_secs(30));
        pw.add_pickup(PickupNote::new("p1", "First pickup"));
        pw.add_pickup(PickupNote::new("p2", "Second pickup"));
        assert_eq!(pw.pickup_count(), 2);
        assert_eq!(pw.pending_pickups().len(), 2);
        assert_eq!(pw.completed_pickups().len(), 0);
    }

    #[test]
    fn test_preparation_window_zero_time() {
        let pw = PreparationWindow::new(Duration::ZERO);
        assert_eq!(pw.usage_fraction(), 1.0);
    }

    #[test]
    fn test_preparation_window_sufficient_time() {
        let mut pw = PreparationWindow::new(Duration::from_secs(10));
        pw.add_pickup(PickupNote::new("p1", "Quick").with_duration(Duration::from_secs(2)));
        pw.add_pickup(PickupNote::new("p2", "Also quick").with_duration(Duration::from_secs(3)));
        assert!(pw.has_sufficient_time());

        pw.use_time(Duration::from_secs(8));
        assert!(!pw.has_sufficient_time());
    }

    #[test]
    fn test_preparation_window_essential_complete() {
        let mut pw = PreparationWindow::new(Duration::from_secs(30));
        let mut essential = PickupNote::new("e1", "Must do").with_criticality(9);
        essential.complete();
        pw.add_pickup(essential);
        pw.add_pickup(PickupNote::new("o1", "Optional").with_criticality(2));
        assert!(pw.essential_pickups_complete());
    }

    // --- AnticipationDetector tests ---

    #[test]
    fn test_anticipation_detector_creation() {
        let ad = AnticipationDetector::new(0.5);
        assert_eq!(ad.threshold(), 0.5);
        assert!(!ad.is_anticipating());
        assert_eq!(ad.signal_count(), 0);
    }

    #[test]
    fn test_anticipation_detector_weak_signal() {
        let mut ad = AnticipationDetector::new(0.7);
        ad.add_signal(AnticipationDetector::signal("calendar-event", 0.3));
        assert!(!ad.is_anticipating());
    }

    #[test]
    fn test_anticipation_detector_strong_signal() {
        let mut ad = AnticipationDetector::new(0.5);
        ad.add_signal(AnticipationDetector::signal("user-typing", 0.9));
        assert!(ad.is_anticipating());
    }

    #[test]
    fn test_anticipation_detector_combined() {
        let mut ad = AnticipationDetector::new(0.5);
        ad.add_signal(AnticipationDetector::signal("signal-a", 0.3));
        assert!(!ad.is_anticipating());
        ad.add_signal(AnticipationDetector::signal("signal-b", 0.8));
        assert!(ad.is_anticipating()); // Combined should push past threshold
    }

    #[test]
    fn test_anticipation_detector_timed_signal() {
        let mut ad = AnticipationDetector::new(0.3);
        ad.add_signal(AnticipationDetector::timed_signal(
            "deadline-approaching",
            0.8,
            Duration::from_secs(60),
        ));
        assert!(ad.is_anticipating());
        assert_eq!(ad.estimated_time_until(), Some(Duration::from_secs(60)));
    }

    #[test]
    fn test_anticipation_detector_clear() {
        let mut ad = AnticipationDetector::new(0.3);
        ad.add_signal(AnticipationDetector::signal("test", 0.9));
        ad.clear();
        assert!(!ad.is_anticipating());
        assert_eq!(ad.signal_count(), 0);
    }

    #[test]
    fn test_anticipation_detector_signal_history() {
        let mut ad = AnticipationDetector::new(0.3);
        ad.add_signal(AnticipationDetector::signal("user-active", 0.7));
        ad.add_signal(AnticipationDetector::signal("user-active", 0.8));
        assert_eq!(*ad.signal_history().get("user-active").unwrap(), 2);
    }

    #[test]
    fn test_anticipation_combined_strength_empty() {
        let ad = AnticipationDetector::new(0.5);
        assert_eq!(ad.combined_strength(), 0.0);
    }

    // --- PreemptiveAction tests ---

    #[test]
    fn test_preemptive_action_creation() {
        let pa = PreemptiveAction::new("prefetch", "Prefetch user data", "User typically requests data after login");
        assert_eq!(pa.id(), "prefetch");
        assert_eq!(pa.action(), "Prefetch user data");
        assert!(!pa.was_executed());
        assert!(pa.was_needed().is_none());
    }

    #[test]
    fn test_preemptive_action_expected_value() {
        let pa = PreemptiveAction::new("prefetch", "Prefetch", "Likely needed")
            .with_confidence(0.8)
            .with_cost_benefit(1.0, 5.0, 0.5);
        // EV = 0.8*5.0 - 0.2*0.5 - 1.0 = 4.0 - 0.1 - 1.0 = 2.9
        assert!((pa.expected_value() - 2.9).abs() < 0.01);
        assert!(pa.is_worthwhile());
    }

    #[test]
    fn test_preemptive_action_not_worthwhile() {
        let pa = PreemptiveAction::new("risky", "Expensive prefetch", "Unlikely needed")
            .with_confidence(0.1)
            .with_cost_benefit(10.0, 5.0, 8.0);
        // EV = 0.1*5.0 - 0.9*8.0 - 10.0 = 0.5 - 7.2 - 10.0 = -16.7
        assert!(pa.expected_value() < 0.0);
        assert!(!pa.is_worthwhile());
    }

    #[test]
    fn test_preemptive_action_execute_and_resolve() {
        let mut pa = PreemptiveAction::new("test", "Test action", "Testing");
        pa.execute();
        assert!(pa.was_executed());
        pa.resolve(true);
        assert_eq!(pa.was_needed(), Some(true));
    }

    #[test]
    fn test_preemptive_action_was_not_needed() {
        let mut pa = PreemptiveAction::new("test", "Test action", "Testing");
        pa.execute();
        pa.resolve(false);
        assert_eq!(pa.was_needed(), Some(false));
    }

    // --- SetupVerification tests ---

    #[test]
    fn test_setup_verification_empty() {
        let mut sv = SetupVerification::new();
        let result = sv.verify();
        assert_eq!(result, VerificationResult::Ready);
        assert!(sv.is_verified());
    }

    #[test]
    fn test_setup_verification_all_pass() {
        let mut sv = SetupVerification::new();
        sv.add_check(VerificationCheck::pass("cache-ready"));
        sv.add_check(VerificationCheck::pass("db-connected"));
        sv.add_check(VerificationCheck::pass("config-loaded"));
        let result = sv.verify();
        assert_eq!(result, VerificationResult::Ready);
        assert_eq!(sv.readiness(), 1.0);
        assert_eq!(sv.passing_count(), 3);
        assert_eq!(sv.failing_count(), 0);
    }

    #[test]
    fn test_setup_verification_critical_failure() {
        let mut sv = SetupVerification::new();
        sv.add_check(VerificationCheck::pass("cache-ready"));
        sv.add_check(VerificationCheck::fail("db-connected", "Connection refused").critical());
        let result = sv.verify();
        match result {
            VerificationResult::NotReady { failures } => {
                assert_eq!(failures.len(), 1);
                assert!(failures[0].contains("db-connected"));
            }
            _ => panic!("Expected NotReady"),
        }
    }

    #[test]
    fn test_setup_verification_partial() {
        let mut sv = SetupVerification::new();
        sv.add_check(VerificationCheck::pass("cache-ready"));
        sv.add_check(VerificationCheck::fail("metrics-exporter", "Not critical"));
        let result = sv.verify();
        match result {
            VerificationResult::PartiallyReady { warnings } => {
                assert_eq!(warnings.len(), 1);
            }
            _ => panic!("Expected PartiallyReady"),
        }
        assert!((sv.readiness() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_verification_check_with_description() {
        let check = VerificationCheck::pass("db")
            .with_description("Check database connectivity");
        assert_eq!(check.description, "Check database connectivity");
    }

    #[test]
    fn test_setup_verification_default() {
        let sv = SetupVerification::default();
        assert_eq!(sv.check_count(), 0);
    }

    // --- Anacrusis tests ---

    #[test]
    fn test_anacrusis_creation() {
        let a = Anacrusis::new("process-user-request", Duration::from_secs(5));
        assert_eq!(a.main_action(), "process-user-request");
        assert!(!a.is_complete());
        // Empty anacrusis is vacuously prepared (no requirements)
        assert!(a.is_prepared());
    }

    #[test]
    fn test_anacrusis_full_flow() {
        let mut a = Anacrusis::new("handle-request", Duration::from_secs(10));

        // Add anticipation
        a.add_signal(AnticipationDetector::signal("user-activity", 0.8));
        assert!(a.detector().is_anticipating());

        // Add pickup notes
        a.add_pickup(PickupNote::new("cache", "Warm cache").with_criticality(9));
        a.add_pickup(PickupNote::new("preload", "Preload data").with_criticality(5));
        assert_eq!(a.window().pickup_count(), 2);

        // Begin preparation
        a.begin_preparation();

        // Simulate completing pickups
        a.window_mut().pickups[0].start();
        a.window_mut().pickups[0].complete();
        a.window_mut().pickups[1].start();
        a.window_mut().pickups[1].complete();

        // Add verification checks
        a.add_check(VerificationCheck::pass("cache-warm"));
        a.add_check(VerificationCheck::pass("data-loaded"));

        // Finalize
        let result = a.finalize();
        assert_eq!(result, VerificationResult::Ready);
        assert!(a.is_complete());
    }

    #[test]
    fn test_anacrusis_not_ready() {
        let mut a = Anacrusis::new("task", Duration::from_secs(5));
        a.begin_preparation();
        a.add_check(VerificationCheck::fail("critical-dep", "Not available").critical());
        let result = a.finalize();
        match result {
            VerificationResult::NotReady { .. } => {}
            _ => panic!("Expected NotReady"),
        }
    }

    #[test]
    fn test_anacrusis_prepared_check() {
        let a = Anacrusis::new("task", Duration::from_secs(5));
    }

    #[test]
    fn test_anacrusis_mutable_access() {
        let mut a = Anacrusis::new("task", Duration::from_secs(5));
        a.window_mut().open();
        assert!(a.window().is_active());
        a.detector_mut().add_signal(AnticipationDetector::signal("test", 0.9));
        assert!(a.detector().is_anticipating());
    }
}
