#[allow(dead_code)]

use std::sync::{Arc, Mutex};

pub type CommandExecutorHandle = Arc<Mutex<dyn CommandExecutor>>;

#[cfg(test)]
use mockall::automock;
use tauri::{command, State};

#[cfg_attr(test, automock)]
pub trait CommandExecutor: Send + Sync + 'static {
  fn execute<'a>(&self, command: &str, args: &[&'a str]) -> Result<(), String>;
}

pub struct DefaultCommandExecutor;

impl CommandExecutor for DefaultCommandExecutor {
    fn execute(&self, command: &str, args: &[&str]) -> Result<(), String> {
        use std::process::Command;

        Command::new(command)
            .args(args)
            .spawn()
            .map_err(|e| format!("Failed to execute command: {}", e))?
            .wait()
            .map_err(|e| format!("Command was terminated with an error: {}", e))?;

        Ok(())
    }
}

impl DefaultCommandExecutor {
    pub fn new() -> Self {
        Self {}
    }
}


#[command]
fn execute_command(state: State<'_, CommandExecutorHandle>) -> Result<(), String> {
  
	let command_executor = state.lock().unwrap();

	command_executor.execute("test", &vec![]).unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	pub fn to_state_unsafe<'r, T: Send + Sync + 'static>(input: &'r T) -> State<'r, T> {
		struct FakeTauriState<'r, T: Send + Sync + 'static>(&'r T);
	
		let fake_state = FakeTauriState(input);
		unsafe { std::mem::transmute(fake_state) }
	}

	#[test]
	fn should_execute_command() {

		let mut command_executor = MockCommandExecutor::new();

		command_executor
			.expect_execute()
			.return_const(Ok(()));

		let command_executor: CommandExecutorHandle = Arc::new(Mutex::new(command_executor));
		let state: State<'_, CommandExecutorHandle> = to_state_unsafe(&command_executor);

		execute_command(state).unwrap();
	}
}