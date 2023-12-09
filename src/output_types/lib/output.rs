pub trait Output {
    fn write(&mut self, src: &str);
}

pub struct MockConsole {
    pub write_was_called: usize,
}
impl Output for MockConsole {
    fn write(&mut self, _src: &str) {
        self.write_was_called += 1;
    }
}

pub struct Console {}

impl Output for Console {
    fn write(&mut self, src: &str) {
        print!("{}", src);
    }
}
