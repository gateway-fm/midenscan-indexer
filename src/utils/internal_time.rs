// introduce a concept of internal time
// used to organize ordering within a block
// {blockTimestamp: 10 digits} { some info: 10 digits} { some info: 10 digits }

// blockTimestamps are unix timestamp in seconds (miden standard)
// - should not exceed 10 numbers anytime soon
//
// max value should be
// 10 digits + 10 digits + 10 digits = 30 digits

pub fn get_internal_time(block_number: u32, value_one: u32, value_two: u32) -> u128 {
    let block_number_str = format!("{:0>10}", block_number);
    let value_one_str = format!("{:0>10}", value_one);
    let value_two_str = format!("{:0>10}", value_two);

    format!("{}{}{}", block_number_str, value_one_str, value_two_str)
        .parse::<u128>()
        .unwrap()
}
