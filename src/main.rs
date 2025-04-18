use grid_builder::build_grid;

mod db_builder;
mod grid_builder;

#[tokio::main]
async fn main() {
    // match build_db().await {
    //     Err(e)=>{dbg!(&e);},
    //     Ok(_) => {print!("success\n");}
    // };
    dbg!(build_grid());
}