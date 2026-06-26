use crate::errors::StepExecutionError;

pub fn execute(arg: &str) -> Result<(), StepExecutionError> {
    println!("{}", arg);

    Ok(())
}
