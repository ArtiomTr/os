use crate::consts::{BLOCK_SIZE, WORD_SIZE};

pub type Program = [u8; BLOCK_SIZE * WORD_SIZE + // program parameters
    7 * BLOCK_SIZE * WORD_SIZE + // program data
    6 * BLOCK_SIZE * WORD_SIZE + // program code
    2 * BLOCK_SIZE * WORD_SIZE];

pub struct VirtualMachine {
    program: Program,
    program_counter_register: u16,
    general_use_register: u32,
    conditional_register: u8,
    stack_pointer_register: u8,
}

impl VirtualMachine {
    pub fn new(program: Program) -> Self {
        Self {
            program,
            program_counter_register: 0,
            general_use_register: 0,
            conditional_register: 0,
            stack_pointer_register: 0,
        }
    }

    pub fn advance(&mut self) {
        let command: [u8; WORD_SIZE] = self.program[8 * BLOCK_SIZE * WORD_SIZE
            + self.program_counter_register as usize * WORD_SIZE
            ..8 * BLOCK_SIZE * WORD_SIZE
                + (self.program_counter_register + 1) as usize * WORD_SIZE]
            .try_into()
            .unwrap();

        match command {
            [b'L', b'R', x, y] => {
                let x = x as usize;
                let y = y as usize;
                self.general_use_register = u32::from_be_bytes(
                    self.program
                        [((x * BLOCK_SIZE + y) * WORD_SIZE)..(x * BLOCK_SIZE + y + 1) * WORD_SIZE]
                        .try_into()
                        .unwrap(),
                );
                self.program_counter_register += 1;
            }
            [b'S', b'R', x, y] => {
                let x = x as usize;
                let y = y as usize;

                self.program
                    [((x * BLOCK_SIZE + y) * WORD_SIZE)..(x * BLOCK_SIZE + y + 1) * WORD_SIZE]
                    .copy_from_slice(&self.general_use_register.to_be_bytes());

                self.program_counter_register += 1;
            }
            [b'A', b'D', x, y] => {
                let x = x as usize;
                let y = y as usize;
                // TODO: update conditional flags
                self.general_use_register =
                    self.general_use_register.wrapping_add(u32::from_be_bytes(
                        self.program[((x * BLOCK_SIZE + y) * WORD_SIZE)
                            ..(x * BLOCK_SIZE + y + 1) * WORD_SIZE]
                            .try_into()
                            .unwrap(),
                    ));
                self.program_counter_register += 1;
            }
            [b'S', b'U', x, y] => {
                let x = x as usize;
                let y = y as usize;
                // TODO: update conditional flags
                self.general_use_register =
                    self.general_use_register.wrapping_sub(u32::from_be_bytes(
                        self.program[((x * BLOCK_SIZE + y) * WORD_SIZE)
                            ..(x * BLOCK_SIZE + y + 1) * WORD_SIZE]
                            .try_into()
                            .unwrap(),
                    ));
                self.program_counter_register += 1;
            }
            [b'C', b'R', _x, _y] => {
                todo!("compare command")
            }
            [b'J', b'P', x, y] => {
                let x = x as usize;
                let y = y as usize;
                // TODO: don't panic
                self.program_counter_register = u32::from_be_bytes(
                    self.program
                        [((x * BLOCK_SIZE + y) * WORD_SIZE)..(x * BLOCK_SIZE + y + 1) * WORD_SIZE]
                        .try_into()
                        .unwrap(),
                )
                .try_into()
                .unwrap();
            }
            [b'J', b'B', _x, _y] => {
                todo!("jump below command")
            }
            [b'H', b'A', b'L', b'T'] => todo!("halting"),
            command => panic!(
                "command {:?} is not recognized, position {}",
                command, self.program_counter_register
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read, path::PathBuf};

    use crate::consts::WORD_SIZE;

    use super::{Program, VirtualMachine};

    #[test]
    fn sample_program() {
        let mut vm = {
            let mut file = File::open(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("programs")
                    .join("test.bin"),
            )
            .expect("Failed to open test program from `programs/test.bin`");

            let mut buffer: Program = [0u8; 1024];
            file.read_exact(&mut buffer)
                .expect("Failed to read test program - program length bust be exactly 1024 bytes");

            VirtualMachine::new(buffer)
        };

        vm.advance(); // Execute LR00, which copies word from memory address 0x0 (value 1) into general use register
        assert_eq!(vm.general_use_register, 1);
        vm.advance(); // Execute AD01, which adds word from memory address 0x4 (value 2) into general use register
        assert_eq!(vm.general_use_register, 3);
        vm.advance(); // Execute SR03, which copies word from general use register into memory address 0x12
        assert_eq!(vm.program[3 * WORD_SIZE..4 * WORD_SIZE], [0, 0, 0, 3]);
        vm.advance(); // Execute SB03, which subtracts word from memory address 0x12 (value 3) from general use register
        assert_eq!(vm.general_use_register, 0);
    }
}
