pub fn shallow_parse_data(line: String) -> (u128, String) {
    let data: Vec<&str> = line.split(":").collect();
    let epoch = data[0].parse::<u128>().unwrap();
    return (epoch, line);
}
