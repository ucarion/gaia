use std::path::Path;
use std::process::{Command, Stdio};
use std::str::FromStr;

use errors::*;

#[derive(Debug)]
pub struct Convert {
    command: Command,
}

impl Convert {
    pub fn new() -> Convert {
        let mut command = Command::new("convert");
        command.stderr(Stdio::inherit());

        Convert { command }
    }

    pub fn monitor(&mut self) -> &mut Convert {
        self.command.arg("-monitor");
        self
    }

    pub fn grayscale_input(&mut self, size: (u32, u32), path: &Path) -> &mut Convert {
        let size_arg = format!("{}x{}", size.0, size.1);
        self.command.args(&["-depth", "16", "-size", &size_arg]);
        self.command.arg(format!("gray:{}", path.to_string_lossy()));
        self
    }

    pub fn input(&mut self, path: &Path) -> &mut Convert {
        self.command.arg(path);
        self
    }

    pub fn adjoin(&mut self) -> &mut Convert {
        self.command.arg("+adjoin");
        self
    }

    pub fn output(&mut self, path: &Path) -> &mut Convert {
        self.command.arg(path);
        self
    }

    pub fn depth(&mut self, depth: u8) -> &mut Convert {
        self.command.args(&["-depth", &format!("{}", depth)]);
        self
    }

    pub fn group<F>(&mut self, f: F) -> &mut Convert
    where
        F: FnOnce(&mut Convert) -> &mut Convert,
    {
        self.command.arg("(");
        f(self);
        self.command.arg(")");
        self
    }

    pub fn crops(&mut self, size: u32) -> &mut Convert {
        let crop_fmt = format!("{}x{}", size, size);

        self.command.args(&["-crop", &crop_fmt]);
        self
    }

    pub fn crop_one(&mut self, size: (u32, u32), offset: (u32, u32)) -> &mut Convert {
        let crop_fmt = format!("{}x{}+{}+{}", size.0, size.1, offset.0, offset.1);

        self.command.args(&["-crop", &crop_fmt]);
        self
    }

    pub fn append_horizontally(&mut self) -> &mut Convert {
        self.command.arg("+append");
        self
    }

    pub fn append_vertically(&mut self) -> &mut Convert {
        self.command.arg("-append");
        self
    }

    pub fn resize(&mut self, resize: &str) -> &mut Convert {
        self.command.args(&["-resize", resize]);
        self
    }

    pub fn offset_each_pixel(&mut self, offset: u16) -> &mut Convert {
        self.command
            .args(&["-evaluate", "addmodulus", &offset.to_string()]);
        self
    }

    pub fn report_max_min(&mut self) -> &mut Convert {
        self.command
            .args(&["-format", "%[fx:minima] %[fx:maxima]", "-write", "info:-"]);
        self
    }

    // convert assets/generated/tiles/0_0_0.pgm -format '%[fx:p{129,21239} * QuantumRange]' info:
    pub fn report_value_at_point(&mut self, point: (u32, u32)) -> &mut Convert {
        let fx_format = format!("%[fx:p{{{},{}}} * QuantumRange]", point.0, point.1);

        self.command.args(&["-format", &fx_format, "info:-"]);
        self
    }

    pub fn run(&mut self) -> Result<String> {
        let output = self.command
            .output()
            .chain_err(|| "Error when running `convert`")?;

        if !output.status.success() {
            return Err("`convert` returned with non-zero exit status".into());
        }

        String::from_utf8(output.stdout).chain_err(|| "Error parsing output as UTF-8")
    }

    pub fn run_with_max_min(&mut self) -> Result<(f32, f32)> {
        let output = self.run()?;

        let output_parts: Vec<_> = output.split(" ").collect();
        let min = f32::from_str(output_parts[0]).unwrap();
        let max = f32::from_str(output_parts[1]).unwrap();
        Ok((min, max))
    }

    pub fn run_with_value(&mut self) -> Result<u16> {
        let output = self.run()?;
        let value = u16::from_str(&output).unwrap();
        Ok(value)
    }
}
