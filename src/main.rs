use std::io::{self, BufReader, BufRead, Read};
use std::collections::HashMap;
use std::vec::Vec;

/// The `BrainfuckOp` type.
enum BrainfuckOp {
    IncrementValueOp, /// +
    DecrementValueOp, /// -
    IncrementPtrOp,   /// >
    DecrementPtrOp,   /// <
    PrintOp,          /// .
    ReadOp,           /// ,
    LoopStartOp,      /// [
    LoopEndOp,        /// ]
    MonoStateOp
}

/// Brainfuck virtual machine status
struct BrainfuckVMStatus {
    /// virtual infinity length tape
    tape: HashMap<i32, i32>,
    /// current cell of the tape
    tape_ptr: i32,
    /// used for keeping track of all valid brainfuck_op
    instruction: Vec<char>,
    /// current brainfuck_op index
    instruction_ptr_current: i32,
    /// keeping track of loops
    instruction_loop_ptr: Vec<i32>,
    
    /// flag of skipping loop, e.g
    /// +-[[[------------++++++++++-.>>[>]>>>--<<<<<<--]]]++++
    ///   ^skipping from, but we need all                ^end of skipping
    ///      instructions inside.
    jump_loop: i32
}

/// Returns a new brainfuck VM status.
///
/// # Example
///
/// ```
/// let status = new_brainfuck_status();
/// ```
fn new_brainfuck_status() -> BrainfuckVMStatus {
    BrainfuckVMStatus {
        tape: HashMap::new(),
        tape_ptr: 0,
        instruction: Vec::new(),
        instruction_ptr_current: -1,
        instruction_loop_ptr: Vec::new(),
        jump_loop: 0
    }
}

/// Returns next corresponding BrainfuckOp of given `character`.
///
/// # Arguments
///
/// * `status`    - A mutable var that holds the status of current brainfuck VM status
/// * `character` - char type op (may be invalid brainfuck op)
/// * `via_loop`  - Due to the way I wrote, a flag is needed to avoid re-adding ops 
///
/// # Example
///
/// ```
/// let status = new_brainfuck_status();
/// let next_op = next_op(&mut status, '+', false);
/// ```
fn next_op(status: &mut BrainfuckVMStatus, character: char, via_loop: bool) -> BrainfuckOp {
    // match BrainfuckOp for character
    let op = match character {
        '+' => BrainfuckOp::IncrementValueOp,
        '-' => BrainfuckOp::DecrementValueOp,
        '>' => BrainfuckOp::IncrementPtrOp,
        '<' => BrainfuckOp::DecrementPtrOp,
        '.' => BrainfuckOp::PrintOp,
        ',' => BrainfuckOp::ReadOp,
        '[' => BrainfuckOp::LoopStartOp,
        ']' => BrainfuckOp::LoopEndOp,
        // invaild char for brainfuck
        // monostate is returned
        _   => BrainfuckOp::MonoStateOp,
    };
    
    // do not append the char_op if we're retriving the next op inside a loop_op
    if !via_loop {
        match op {
            BrainfuckOp::MonoStateOp => (),
            _ => {
                // save char_op to instruction
                status.instruction.push(character);
                // increse the ptr of current instruction
                status.instruction_ptr_current += 1;
            }
        };
    }
    // return next op
    op
}

/// Run brainfuck VM
///
/// # Arguments
///
/// * `status`    - A mutable var that holds the status of current brainfuck VM status
/// * `char_op`   - char type op (may be invalid brainfuck op)
/// * `via_loop`  - Due to the way I wrote, a flag is needed to avoid re-adding ops 
///
/// # Example
///
/// ```
/// let status = new_brainfuck_status();
/// run_vm(&mut status, '+', false);
/// ```
fn run_vm(status: &mut BrainfuckVMStatus, char_op: char, via_loop: bool) {
    // get next op from char_op
    let op = next_op(status, char_op, via_loop);
    match op {
        BrainfuckOp::IncrementValueOp => {
            // skip actual action if we're skipping loop
            if status.jump_loop == 0 {
                let count = status.tape.entry(status.tape_ptr).or_insert(0);
                *count += 1;
            }
        },
        BrainfuckOp::DecrementValueOp => {
            // skip actual action if we're skipping loop
            if status.jump_loop == 0 {
                let count = status.tape.entry(status.tape_ptr).or_insert(0);
                *count -= 1;
            }
        },
        BrainfuckOp::IncrementPtrOp => {
            // skip actual action if we're skipping loop
            if status.jump_loop == 0 {
                status.tape_ptr += 1;
            }
        },
        BrainfuckOp::DecrementPtrOp => {
            // skip actual action if we're skipping loop
            if status.jump_loop == 0 {
                status.tape_ptr -= 1;
            }
        },
        BrainfuckOp::PrintOp => {
            // skip actual action if we're skipping loop
            if status.jump_loop == 0 {
                // take cell from tape
                let out = *status.tape.entry(status.tape_ptr).or_insert(0);
                // print as char
                print!("{}", (out % 255) as u8 as char);
            }
        },
        BrainfuckOp::ReadOp => {
            // skip actual action if we're skipping loop
            if status.jump_loop == 0 {
                // Rust read a single char from stdin
                // https://stackoverflow.com/a/30679861
                let input: Option<i32> = std::io::stdin()
                    .bytes() 
                    .next()
                    .and_then(|result| result.ok())
                    .map(|byte| byte as i32);
                // store in tape
                let out = status.tape.entry(status.tape_ptr).or_insert(0);
                *out = input.unwrap();
            }
        },
        BrainfuckOp::LoopStartOp => {
            // if and only if 1) `current_cell_value != 0`
            //                2) and we're not do the skipping
            // we can record the starting index of the if instruction
            // besides, if we're in condition 1)
            // the if statement should be also skipped
            let cell = status.tape.entry(status.tape_ptr).or_insert(0);
            if *cell != 0 && status.jump_loop == 0 {
                status.instruction_loop_ptr.push(status.instruction_ptr_current);
            } else {
                status.jump_loop += 1;
            }
        },
        BrainfuckOp::LoopEndOp => {
            // decrease the jump_loop value if we encounter the `]`
            // and we were previously doing the skip
            if status.jump_loop != 0 {
                status.jump_loop -= 1;
            } else {
                // if we were not in skipping
                // then we need to check the loop condition, `current_cell_value != 0`
                let cell = *status.tape.entry(status.tape_ptr).or_insert(0);
                if cell != 0 {
                    // loop the instruction until condition satisfies no more
                    while *status.tape.entry(status.tape_ptr).or_insert(0) != 0 {
                        // save current instruction pointer
                        let current = status.instruction_ptr_current;
                        // start the loop right after the index of `[`
                        if let Some(last) = status.instruction_loop_ptr.last().cloned() {
                            status.instruction_ptr_current = last + 1;
                            while status.instruction_ptr_current < current {
                                // run one op at a time
                                // until the next op is the corresponding `]`
                                let current_op = status.instruction[status.instruction_ptr_current as usize];
                                run_vm(status, current_op, true);
                                status.instruction_ptr_current += 1;
                            }
                            // restore the current instruction pointer
                            status.instruction_ptr_current = current;
                        }
                    }
                    // pop current loop starting index
                    status.instruction_loop_ptr.pop();
                }
            }
        },
        BrainfuckOp::MonoStateOp => ()
    }
}

fn main() -> io::Result<()> {
    // the brainfuck vm
    let mut status = new_brainfuck_status();
    
    // read from stdin
    let buffer = BufReader::new(io::stdin());
    for line in buffer.lines() {
        for c in line?.chars() {
            // handle every character
            run_vm(&mut status, c, false);
        }
    }
    Ok(())
}
