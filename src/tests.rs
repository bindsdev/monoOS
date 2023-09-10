pub struct Test {
    pub path: &'static str,
    pub func: fn(),
    pub quiet: bool,
}

pub fn test_runner(tests: &[&Test]) {
    log::info!("running {} test(s)", tests.len());

    let mut passed = 0;
    let mut suppressed = 0;

    for test in tests {
        (test.func)();

        if !test.quiet {
            log::info!("test {} ... ok", test.path);
        } else {
            suppressed += 1;
        }

        passed += 1;
    }

    log::info!(
        "tests completed: {} passed; {} suppressed",
        passed,
        suppressed
    );
}
