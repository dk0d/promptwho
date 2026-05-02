// TODO: This is a placeholder for the actual implementation of the promptwho-bootstrap crate.
//
// This crate will facilitate the cli in installing the opencode plugin and any other configuration
// for the promptwho server
//
// - Plugin installation and config
// - Server config - local server or remote server
// - DB Config
//      - DB flavor (surreal, sql, etc.)
//      - Vector DB features etc.
//
// Intention is to have a default initializer, but also have an interactive prompting flow for the user
// on first spin up via the CLI tooling
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
