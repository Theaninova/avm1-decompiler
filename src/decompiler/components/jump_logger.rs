use crate::ast::expr::Expression;

pub fn log_jump(
    offset: i16,
    position: usize,
    actual_position: usize,
    target: usize,
    condition: &Option<Expression>,
) {
    let jump = if offset < 0 {
        format!(
            ">> {:04} <- [{:04}-{:04}]",
            target, position, actual_position
        )
    } else {
        format!(
            ">> [{:04}-{:04}] -> {:04}",
            position, actual_position, target
        )
    };

    if let Some(condition) = condition {
        println!("{} if not {}", jump, condition);
    } else {
        println!("{}", jump)
    }
}

pub fn log_return(position: usize, actual_position: usize, value: &Option<Expression>) {
    if let Some(value) = value {
        println!(
            ">> [{:04}-{:04}] return {}",
            position, actual_position, value,
        );
    } else {
        println!(">> [{:04}-{:04}] return", position, actual_position,);
    }
}
