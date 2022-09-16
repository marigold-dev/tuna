use std::any::type_name;

use crate::errors::{vm::VmError, VMResult};

pub fn to_i64<T: TryInto<i64> + ToString + Copy>(input: T) -> VMResult<i64> {
    input.try_into().map_err(|_| {
        let error = format!(
            "{:?}, {:?}. {}",
            type_name::<T>(),
            type_name::<i64>(),
            input.to_string(),
        );
        VmError::RuntimeErr(error)
    })
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
