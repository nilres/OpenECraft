use super::*;

fn finished<T: Send + 'static>(job: &Job<T>) -> Result<T, String> {
    let start = std::time::Instant::now();
    loop {
        if let Some(r) = job.take() {
            return r;
        }
        assert!(start.elapsed().as_secs() < 30, "the job does not finish");
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}

#[test]
fn the_result_is_taken_once() {
    let ctx = egui::Context::default();
    let job = Job::start("adding".into(), &ctx, || 2 + 3);
    assert_eq!(job.label, "adding");
    assert_eq!(finished(&job), Ok(5));
    assert!(job.take().is_none());
}

#[test]
fn a_failure_is_an_error() {
    let ctx = egui::Context::default();
    let job: Job<u32> = Job::start("failing".into(), &ctx, || panic!("broken file"));
    assert_eq!(finished(&job), Err("broken file".to_string()));
}
