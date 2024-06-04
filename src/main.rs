use dotenv::dotenv;

pub mod models;
pub mod repository;
pub mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenv().ok();        

	let post_repository: Box<dyn repository::Post> = Box::new(
		repository::postgresql::Post::new()
	);

	let posts = post_repository.list(None);

	println!("--------\nPostgres posts: {:#?}", posts);	

	let post_repository = repository::mongodb::Post::new();

	let user_id = "aaaaaaaaaaaaaaaaaaaaaaaa";	

	let client = repository::mongodb::utils::connect()?;
	let mut session = client.start_session(None)?;

	session.start_transaction(None)?;	

	let posts = post_repository.liked_list_ws(user_id, &mut session);

	println!("{:#?}", posts);

	session.abort_transaction()?;

	Ok(())
}
