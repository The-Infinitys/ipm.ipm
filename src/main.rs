use cmd_arg::cmd_arg;
use ipak::dprintln;
fn main() -> Result<(), std::io::Error> {
    let command_data = cmd_arg::get();
    dprintln!("{}", command_data);
    let opts = command_data.opts;

    // 引数がない場合は早期リターン
    if opts.is_empty() {
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
    }

    // let command = &opts[0];
    // let sub_opts: Vec<&cmd_arg::Option> = opts[1..].iter().collect();

    println!("Hello, world!");
    Ok(())
}
