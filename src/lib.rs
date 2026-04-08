/// Demonstrates the project structure.
pub struct Greeter {
    pub name: String,
}

impl Greeter {
    /// Create a new Greeter.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    /// Return a greeting message.
    pub fn greet(&self) -> String {
        format!("Hello, {}!", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        let g = Greeter::new("World");
        assert_eq!(g.greet(), "Hello, World!");
    }

    #[test]
    fn test_greet_custom_name() {
        let g = Greeter::new("Agent");
        assert_eq!(g.greet(), "Hello, Agent!");
    }
}
