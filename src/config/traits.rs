/// The trait for describing a channel
pub trait ChannelConfig {
    /// human-readable name
    fn name() -> &'static str;
    /// short description
    fn desc() -> &'static str;
}

// Maybe there should be a `&self` as parameter for custom channel/info or what...

pub trait ConfigHandle {
    fn name(&self) -> &'static str;
    fn desc(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestChannelConfig;
    impl ChannelConfig for TestChannelConfig {
        fn name() -> &'static str {
            "test-channel"
        }
        fn desc() -> &'static str {
            "A test channel"
        }
    }

    struct TestConfigHandle;
    impl ConfigHandle for TestConfigHandle {
        fn name(&self) -> &'static str {
            "test-handle"
        }
        fn desc(&self) -> &'static str {
            "A test config handle"
        }
    }

    /// REQ-CFG-TRAIT-001-SC01
    #[test]
    fn channel_config_returns_static_name_and_desc() {
        assert_eq!(TestChannelConfig::name(), "test-channel");
        assert_eq!(TestChannelConfig::desc(), "A test channel");
    }

    /// REQ-CFG-TRAIT-002-SC01
    #[test]
    fn config_handle_returns_static_name_and_desc() {
        let handle = TestConfigHandle;
        assert_eq!(handle.name(), "test-handle");
        assert_eq!(handle.desc(), "A test config handle");
    }
}
