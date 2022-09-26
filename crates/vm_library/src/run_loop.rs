use crate::{
    arena::{ARENA, CONSUMEDTICKETS, TICKETABLE},
    compile,
    contract_address::{decode, ContractAddress},
    errors::{vm::VmError, VMResult},
    execution_result::ExecutionResult,
    incoming::InvokeManaged,
    instance::invoke_managed,
    managed::value::Value,
    outgoing::{InitVec, SetOwned},
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
    loop {
        let msg = io.read();
        match msg {
            ClientMessage::SetInitialState(x) => {
                State::from_init(&mut state, x).expect("failed to init_state");
                break;
            }
            ClientMessage::GetInitialState => io.write(&ServerMessage::Init(InitVec(vec![]))),
            ClientMessage::NoopTransaction => io.write(&ServerMessage::Stop),
            x => panic!("init not supported, {:?}", x),
        }
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
                    dbg!("received transaction");

                    match handle_transaction(&mut context, transaction, false, 0) {
                        Ok(_) => context.io.write(&ServerMessage::Stop),
                        Err(_) => break 'inner,
                    }
                }
                ClientMessage::NoopTransaction => {
                    dbg!("received noop");
                    context.io.write(&ServerMessage::Stop);
                    break 'inner;
                }
                x => panic!("run_loop not supported, {:?}", x),
            }
            context.to_revert.clear();
        }
    }
}

fn handle_transaction(
    context: &mut ExecutionState,
    transaction: Transaction,
    get_tickets: bool,
    mut gas_limit: u64,
) -> VMResult<u64> {
    let io = &mut context.io;

    if let Ok(op) = serde_json::from_value(transaction.operation.clone()) {
        match op {
            Operation::Invoke {
                address,
                argument,
                gas_limit: op_gas_limit,
            } => {
                gas_limit = op_gas_limit;
                let mut tickets2: Vec<Ticket> = transaction
                    .tickets
                    .clone()
                    .into_iter()
                    .map(|(x, y)| Ticket::new(x, y))
                    .collect();
                if get_tickets {
                    context
                        .io
                        .write_with_fail(&ServerMessage::TakeTickets(address.clone()))
                        .map_err(|err| VmError::RuntimeErr(err.to_string()))?;
                    'd: loop {
                        match context.io.read() {
                            ClientMessage::GiveTickets(ticket) => {
                                tickets2.extend(ticket.into_iter().map(|(x, y)| Ticket::new(x, y)));
                                break 'd;
                            }
                            ClientMessage::NoopTransaction => (),
                            _ => panic!("bad format"),
                        }
                    }
                };

                context.ticket_table.populate(&tickets2);
                let new_limit =
                    handle_invoke(context, transaction, address, argument, gas_limit, tickets2)?;
                gas_limit = new_limit;
                Ok(())
            }
            Operation::Originate {
                module_,
                constants,
                initial_storage,
            } => {
                let addres = handle_originate(
                    context,
                    module_,
                    constants,
                    initial_storage,
                    transaction.operation_raw_hash,
                    transaction.source,
                )?;
                let address = contract_addr_to_string(&addres);
                context
                    .io
                    .write_with_fail(&ServerMessage::DepositTickets(TicketDeposit {
                        address,
                        tickets: transaction.tickets,
                    }))
                    .map_err(|err| VmError::RuntimeErr(err.to_string()))
            }
            Operation::Transfer { address, tickets } => context
                .io
                .write_with_fail(&ServerMessage::DepositTickets(TicketDeposit {
                    address,
                    tickets,
                }))
                .map_err(|err| VmError::RuntimeErr(err.to_string())),
        }?;
        Ok::<u64, VmError>(gas_limit)
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
) -> VMResult<ContractAddress> {
    let module = compile::compile_managed_module(module.as_bytes())?;
    let serialized = module
        .serialize()
        .map_err(|x| VmError::CompileErr(x.to_string()))?;
    let addr = ContractAddress::new(operation_hash.as_bytes());
    let contract_type = ContractType {
        self_: addr.clone(),
        originated_by,
        storage: bincode::serialize(&initial_storage).expect("error"),
        module: Box::from(Some(module)),
        serialized_module: serialized,
        constants: bincode::serialize(&constants).expect("error"),
    };
    let serialized = bincode::serialize(&contract_type).unwrap();
    let serialized = &String::from_utf8_lossy(&serialized);
    let msg = &ServerMessage::Set(SetOwned {
        key: addr.clone(),
        value: serialized.to_string(),
    });
    context.state.set(addr.clone(), contract_type);
    match context.io.write_with_fail(msg) {
        Ok(()) => Ok(addr),
        Err(_) => {
            context
                .io
                .write(&ServerMessage::Error("failed to set".to_owned()));
            Err(VmError::RuntimeErr("cant talk to host".to_owned()))
        }
    }
}
pub fn contract_addr_to_string(c: &ContractAddress) -> String {
    c.0.clone()
}
fn handle_invoke(
    context: &mut ExecutionState,
    transaction: Transaction,
    address: ContractAddress,
    argument: Value,
    mut gas_limit: u64,
    tickets: Vec<Ticket>,
) -> VMResult<u64> {
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
                    mod_: (*contract.module).clone().unwrap(),
                    arg,
                    initial_storage,
                    constants: constantst,
                    tickets,
                    source: transaction.source.clone(),
                    sender: transaction
                        .sender
                        .unwrap_or_else(|| transaction.source.clone()),
                    self_addr: address.0.clone(),
                    gas_limit,
                };
                let self_addr = address.clone();
                match invoke_managed(invoke_payload) {
                    Ok(ExecutionResult {
                        new_storage,
                        ops,
                        remaining_gas,
                    }) => {
                        gas_limit = remaining_gas;
                        context.ticket_table.finalize();
                        let serialized_storage =
                            bincode::serialize(&new_storage).expect("serialization_error");
                        {
                            let deposit = unsafe { &mut CONSUMEDTICKETS };
                            let address = contract_addr_to_string(&address);
                            context
                                .io
                                .write(&ServerMessage::DepositTickets(TicketDeposit {
                                    address,
                                    tickets: deposit.to_vec(),
                                }));
                            deposit.clear();
                        };
                        contract.set_storage(serialized_storage);
                        let serialized = bincode::serialize(&contract).unwrap();
                        let serialize = &String::from_utf8_lossy(&serialized);
                        let msg = &ServerMessage::Set(SetOwned {
                            key: address.clone(),
                            value: serialize.to_string(),
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
                        let arena = unsafe { &ARENA };
                        match ops {
                            Value::List(l) if !l.is_empty() => {
                                let res: VMResult<Vec<Transaction>> = l
                                    .into_iter()
                                    .map(|trans| match trans {
                                        Value::Pair { fst, snd } => {
                                            let self_addr = self_addr.clone();

                                            let address =
                                                arena.get(fst).cloned().ok_or_else(|| {
                                                    VmError::RuntimeErr(
                                                "bad transaction format from additional operations"
                                                    .to_owned(),
                                            )
                                                })?;
                                            let address = match address {
                                            Value::String(s) => Ok(s),
                                            _ => Err(VmError::RuntimeErr(
                                                "bad transaction format from additional operations"
                                                    .to_owned(),
                                            )),
                                        }?;

                                            let content =
                                                arena.get(snd).cloned().ok_or_else(|| {
                                                    VmError::RuntimeErr(
                                                "bad transaction format from additional operations"
                                                    .to_owned(),
                                            )
                                                })?;

                                            match decode(address.as_bytes()) {
                                                Ok(_) => {
                                                    let operation = Operation::Invoke {
                                                        address: self_addr.clone(),
                                                        argument: content,
                                                        gas_limit: remaining_gas,
                                                    };
                                                    let deposit = unsafe { &mut CONSUMEDTICKETS };
                                                    let operation = serde_json::to_value(
                                                        &operation,
                                                    )
                                                    .map_err(|err| {
                                                        VmError::RuntimeErr(err.to_string())
                                                    })?;
                                                    let tickets = deposit.clone();
                                                    deposit.clear();
                                                    let transaction = Transaction {
                                                        source: transaction.source.clone(),
                                                        sender: Some(self_addr.0),
                                                        operation,
                                                        operation_raw_hash: transaction
                                                            .operation_raw_hash
                                                            .clone(),
                                                        tickets,
                                                    };

                                                    Ok(transaction)
                                                }
                                                Err(_) => {
                                                    let deposit = unsafe { &mut CONSUMEDTICKETS };
                                                    serde_json::to_string(&content).map_err(
                                                        |err| VmError::RuntimeErr(err.to_string()),
                                                    )?;
                                                    let tickets = deposit.clone();
                                                    deposit.clear();
                                                    let operation = Operation::Transfer {
                                                        address,
                                                        tickets: tickets.clone(),
                                                    };
                                                    let operation = serde_json::to_value(
                                                        &operation,
                                                    )
                                                    .map_err(|err| {
                                                        VmError::RuntimeErr(err.to_string())
                                                    })?;
                                                    let transaction = Transaction {
                                                        source: transaction.source.clone(),
                                                        sender: Some(self_addr.0),
                                                        operation,
                                                        operation_raw_hash: transaction
                                                            .operation_raw_hash
                                                            .clone(),
                                                        tickets,
                                                    };

                                                    Ok(transaction)
                                                }
                                            }
                                        }
                                        _ => Err(VmError::RuntimeErr(
                                            "bad transaction format from additional operations"
                                                .to_owned(),
                                        )),
                                    })
                                    .collect();
                                let res = res?;
                                res.into_iter().try_for_each(|x| {
                                    let transaction = x;
                                    let new_gas = handle_transaction(
                                        context,
                                        transaction,
                                        true,
                                        remaining_gas,
                                    )?;
                                    gas_limit = new_gas;
                                    Ok::<(), VmError>(())
                                })
                            }
                            _ => Ok(()),
                        }?;
                        Ok(gas_limit)
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
