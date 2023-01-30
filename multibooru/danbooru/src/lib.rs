pub mod post;
pub use post::Post;
pub mod record;
pub mod tag;
pub use tag::Tag;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
