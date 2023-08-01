#[derive(Debug, Clone, Default)]
pub struct Console {
	pub stdout: Vec<String>,
	stderr: Vec<String>,
}

impl Console {
	pub fn new(stdout: Vec<String>) -> Self {
		Self { stdout, stderr: Vec::new() }
	}
}
