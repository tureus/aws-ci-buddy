mod cp;
pub use cp::*;

mod ls;
pub use ls::*;

pub fn clean_prefix(prefix: &str) -> Option<String> {
    match prefix {
        "" | "/" => None,
        k => Some(
            k.strip_prefix("/")
                .map(|rest| rest.to_string())
                .unwrap_or_else(|| k.to_string()),
        ),
    }
}
