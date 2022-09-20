use crate::errors::VMResult;

pub fn to_i64(input: u64) -> VMResult<i64> {
    Ok(input as i64)
}

// pub fn ref_to_i64<T: TryInto<i64> + ToString + Clone>(input: &T) -> VMResult<i64> {
//     input.clone().try_into().map_err(|_| {
//         let error = format!(
//             "{}, {}. {}",
//             type_name::<T>(),
//             type_name::<i64>(),
//             input.to_string(),
//         );
//         VmError::RuntimeErr(error)
//     })
// }
#[cfg(test)]
mod test {
    #[test]
    fn example() {
        let max = u64::MAX;
        let conved = max as i64;
        assert_eq!(max, conved as u64)
    }
}
