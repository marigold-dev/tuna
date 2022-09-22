use crate::{
    arena::{ARENA, CONSUMEDTICKETS, TICKETABLE},
    compile,
    contract_address::ContractAddress,
    errors::{vm::VmError, VMResult},
    execution_result::ExecutionResult,
    incoming::InvokeManaged,
    instance::invoke_managed,
    managed::value::Value,
    outgoing::Set,
    pipe::IO,
    state::{ContractType, State},
    ticket_table::{Ticket, TicketTable},
    vm_client::{ClientMessage, Operation, Transaction},
    vm_server::{ServerMessage, TicketDeposit},
};
struct ExecutionState<'a> {
    pub state: State,
    pub to_revert: Vec<(ContractAddress, ContractType)>,
    pub io: IO,
    pub ticket_table: &'a mut TicketTable,
}
pub fn run_loop(mut io: IO) {
    let mut state = State::default();
    let msg = io.read();
    match msg {
        ClientMessage::SetInitialState(x) => {
            State::from_init(&mut state, x).expect("failed to init_state")
        }

        x => panic!("init not supported, {:?}", x),
    }
    let to_revert: Vec<(ContractAddress, ContractType)> = Vec::with_capacity(100);
    let mut context = ExecutionState {
        state,
        to_revert,
        io,
        ticket_table: unsafe { &mut TICKETABLE },
    };
    loop {
        context
            .to_revert
            .drain(0..)
            .for_each(|(addr, contract_type)| {
                context.state.set(addr, contract_type);
            });
        let arena = unsafe { &mut ARENA };

        arena.clear();

        'inner: loop {
            let msg = context.io.read();
            match msg {
                ClientMessage::Transaction(transaction) => {
                    match handle_transaction(&mut context, transaction) {
                        Ok(()) => context.io.write(&ServerMessage::Stop),
                        Err(_) => break 'inner,
                    }
                }
                x => panic!("run_loop not supported, {:?}", x),
            }
            context.to_revert.clear();
        }
    }
}

fn handle_transaction(context: &mut ExecutionState, transaction: Transaction) -> VMResult<()> {
    let io = &mut context.io;
    let tickets: Vec<Ticket> = transaction
        .tickets
        .clone()
        .into_iter()
        .map(|(x, y)| Ticket::new(x, y))
        .collect();
    context.ticket_table.populate(&tickets);
    if let Ok(op) = serde_json::from_str(&transaction.operation) {
        match op {
            Operation::Invoke {
                address,
                argument,
                gas_limit,
            } => handle_invoke(context, transaction, address, argument, gas_limit, tickets),
            Operation::Originate {
                module,
                constants,
                initial_storage,
            } => handle_originate(
                context,
                module,
                constants,
                initial_storage,
                transaction.operation_raw_hash,
                transaction.source,
            ),
        }?;
        Ok::<(), VmError>(())
    } else {
        io.write(&ServerMessage::Error("bad operation".to_owned()));
        Err(VmError::DeserializeErr("Bad transaction".to_owned()))
    }
}
fn handle_originate(
    context: &mut ExecutionState,
    module: String,
    constants: Vec<(u32, Value)>,
    initial_storage: Value,
    operation_hash: String,
    originated_by: String,
) -> VMResult<()> {
    let module = compile::compile_managed_module(module)?;
    let serialized = module
        .serialize()
        .map_err(|x| VmError::CompileErr(x.to_string()))?;
    let addr = ContractAddress::new(operation_hash);
    let contract_type = ContractType {
        self_: addr.clone(),
        originated_by,
        storage: bincode::serialize(&initial_storage).expect("error"),
        module: Some(Box::from(module)),
        serialized_module: serialized,
        constants: bincode::serialize(&constants).expect("error"),
    };
    let serialized = bincode::serialize(&contract_type).unwrap();
    let serialized = &String::from_utf8_lossy(&serialized);
    let msg = &ServerMessage::Set(Set {
        key: &addr,
        value: serialized,
    });
    context.state.set(addr.clone(), contract_type);
    match context.io.write_with_fail(msg) {
        Ok(()) => Ok(()),
        Err(_) => {
            context
                .io
                .write(&ServerMessage::Error("failed to set".to_owned()));
            Err(VmError::RuntimeErr("cant talk to host".to_owned()))
        }
    }
}

fn handle_invoke(
    context: &mut ExecutionState,
    transaction: Transaction,
    address: ContractAddress,
    argument: Value,
    gas_limit: u64,
    tickets: Vec<Ticket>,
) -> VMResult<()> {
    match context.state.get(&address) {
        Some(contract) => {
            context.to_revert.push((address.clone(), contract.clone()));
            {
                let arg = argument;
                let mut contract = contract;
                contract.init()?;
                let initial_storage: Value =
                    bincode::deserialize(&contract.storage).expect("error");
                let constantst: Vec<(i32, Value)> =
                    bincode::deserialize(&contract.constants).expect("error");
                let invoke_payload = InvokeManaged {
                    mod_: *(contract.module.clone().unwrap()),
                    arg,
                    initial_storage,
                    constants: constantst,
                    tickets,
                    source: transaction.source.clone(),
                    sender: transaction.source,
                    self_addr: serde_json::to_string(&address).expect("error"),
                    gas_limit: gas_limit as usize,
                };
                match invoke_managed(invoke_payload) {
                    Ok(ExecutionResult {
                        new_storage,
                        ops,
                        remaining_gas,
                    }) => {
                        context.ticket_table.finalize();
                        let serialized_storage =
                            bincode::serialize(&new_storage).expect("serialization_error");
                        {
                            let deposit = unsafe { &mut CONSUMEDTICKETS };
                            context
                                .io
                                .write(&ServerMessage::DepositTickets(TicketDeposit {
                                    address: address.clone(),
                                    tickets: deposit,
                                }));
                            deposit.clear();
                        };
                        contract.set_storage(serialized_storage);
                        let serialized = bincode::serialize(&contract).unwrap();
                        let serialize = &String::from_utf8_lossy(&serialized);
                        let msg = &ServerMessage::Set(Set {
                            key: &address,
                            value: serialize,
                        });
                        context.state.set(address.clone(), contract);
                        match context.io.write_with_fail(msg) {
                            Ok(()) => (),
                            Err(_) => {
                                context
                                    .io
                                    .write(&ServerMessage::Error("failed to set".to_owned()));
                                return Err(VmError::RuntimeErr("cant talk to host".to_owned()));
                            }
                        };
                        Ok(())
                        // match ops {
                        //     Value::List(l) if !l.is_empty() => l.into_iter().for_each(|x| {
                        //         let op = x;
                        //     }),
                        //     _ => (),
                        // }
                    }
                    Err(x) => {
                        context.io.write(&ServerMessage::Error(x.to_string()));
                        Err(VmError::RuntimeErr("Error_ocured".to_owned()))
                    }
                }
            }
        }
        None => {
            context.io.write(&ServerMessage::Error(format!(
                "contract doesnt exist {}",
                serde_json::to_string(&address).expect("cant happen")
            )));
            Err(VmError::RuntimeErr("Error_ocured".to_owned()))
        }
    }
}
