pub struct Env;

impl Env {
    pub fn new_detect() -> Self {
        Env
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
