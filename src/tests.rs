pub struct Test {
    pub path: &'static str,
    pub func: fn(),
    pub quiet: bool,
}

pub fn test_runner(tests: &[&Test]) {
    log::info!("running {} tests", tests.len());

    let mut successful = 0;
    let mut suppressed = 0;

    for test in tests {
        (test.func)();

        if !test.quiet {
            log::info!("test {} ... ok", test.path);
        } else {
            suppressed += 1;
        }
    }

    log::info!("");
    log::info!(
        "tests completed. {} successful; {} suppressed",
        successful,
        suppressed
    );
}

#[cfg(test)]
mod tests {
    #[mono_proc::test]
    fn testtest() {
        assert!(true);
    }
}
