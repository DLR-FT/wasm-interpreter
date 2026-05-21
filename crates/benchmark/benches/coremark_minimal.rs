fn main() {
    let score_by_runner = benchmark::coremark_minimal::run();

    println!("Runtime: Score (higher is better)");
    for (runner, score) in score_by_runner {
        println!("{}: {}", runner, score)
    }
}
