
#[tokio::main]
async fn main() {
    use tokio::time;

    async fn task_that_takes_a_second() {
        println!("hello");
        time::sleep(time::Duration::from_secs(5)).await
    }

    let mut interval = time::interval(time::Duration::from_secs(1));
    for _i in 0..5 {
        interval.tick().await;
        task_that_takes_a_second().await;
    }
}
