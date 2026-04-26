/// Provider environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Environment {
    /// Sandbox or test mode.
    Sandbox,
    /// Production or live mode.
    Production,
}
