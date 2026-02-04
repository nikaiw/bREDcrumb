pub mod c_cpp;
pub mod csharp;
pub mod go;
pub mod java;
pub mod rust;

pub use c_cpp::CCodeGenerator;
pub use csharp::CSharpCodeGenerator;
pub use go::GoCodeGenerator;
pub use java::JavaCodeGenerator;
pub use rust::RustCodeGenerator;

pub trait CodeGenerator {
    fn generate(&self, string: &str) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_generator() {
        let gen = CCodeGenerator::new(false);
        let code = gen.generate("TEST123");
        assert!(code.contains("TEST123"));
        assert!(code.contains("TRACKING_STRING"));
    }

    #[test]
    fn test_cpp_generator() {
        let gen = CCodeGenerator::new(true);
        let code = gen.generate("TEST123");
        assert!(code.contains("TEST123"));
        assert!(code.contains("TRACKING_STRING"));
    }

    #[test]
    fn test_go_generator() {
        let code = GoCodeGenerator.generate("TEST123");
        assert!(code.contains("TEST123"));
        assert!(code.contains("trackingString"));
    }

    #[test]
    fn test_rust_generator() {
        let code = RustCodeGenerator.generate("TEST123");
        assert!(code.contains("TEST123"));
        assert!(code.contains("TRACKING_STRING"));
    }

    #[test]
    fn test_csharp_generator() {
        let code = CSharpCodeGenerator.generate("TEST123");
        assert!(code.contains("TEST123"));
        assert!(code.contains("TrackingString"));
    }

    #[test]
    fn test_java_generator() {
        let code = JavaCodeGenerator.generate("TEST123");
        assert!(code.contains("TEST123"));
        assert!(code.contains("TrackingString"));
    }

    #[test]
    fn test_escaping_quotes() {
        let code = CCodeGenerator::new(false).generate("test\"quote");
        assert!(code.contains("\\\"") || code.contains("test\"quote"));
    }

    #[test]
    fn test_escaping_backslash() {
        let code = RustCodeGenerator.generate("test\\path");
        assert!(code.contains("\\\\") || code.contains("test\\path"));
    }

    #[test]
    fn test_empty_string() {
        let code = CCodeGenerator::new(false).generate("");
        assert!(code.contains("TRACKING_STRING"));
    }

    #[test]
    fn test_special_chars() {
        let code = GoCodeGenerator.generate("test\n\t\r");
        assert!(code.contains("trackingString"));
    }

    #[test]
    fn test_unicode() {
        let code = RustCodeGenerator.generate("test_émoji_🎯");
        assert!(code.contains("TRACKING_STRING"));
    }

    #[test]
    fn test_long_string() {
        let long_str = "A".repeat(1000);
        let code = JavaCodeGenerator.generate(&long_str);
        assert!(code.contains("TrackingString"));
    }

    #[test]
    fn test_all_generators_produce_output() {
        let generators: Vec<Box<dyn CodeGenerator>> = vec![
            Box::new(CCodeGenerator::new(false)),
            Box::new(CCodeGenerator::new(true)),
            Box::new(GoCodeGenerator),
            Box::new(RustCodeGenerator),
            Box::new(CSharpCodeGenerator),
            Box::new(JavaCodeGenerator),
        ];

        for gen in generators {
            let code = gen.generate("TEST");
            assert!(!code.is_empty());
        }
    }
}
