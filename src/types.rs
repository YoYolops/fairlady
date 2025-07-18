pub struct InternalError {
    pub kind: InternalErrorKind,
    pub message: String,
    stack_trace: Vec<String>
}

impl InternalError {
    pub fn new(kind: InternalErrorKind, message: String) -> Self {
        Self {
            kind,
            message,
            stack_trace: Vec::new()
        }
    }

    pub fn sign_stack_trace(&mut self, signature: String) {
        self.stack_trace.push(signature);
    }

    pub fn print_stack_trace(&self) {
        for signature in &self.stack_trace {
            println!("-- {}", signature);
        }
    }
}

pub enum InternalErrorKind {
    TcpConnection
}

impl InternalErrorKind {
    pub fn to_string(&self) -> String {
        match self {
            InternalErrorKind::TcpConnection => String::from("TcpConnection"),
        }
    }
}
